// Сохранение и восстановление D3D11 pipeline state вокруг egui рендера
//
// egui-directx11 устанавливает свой pipeline state но НЕ восстанавливает старый
// Мы обязаны это сделать чтобы не сломать рендер игры

use std::mem::ManuallyDrop;
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY;
use windows::Win32::Graphics::Direct3D11::*;


pub(crate) unsafe fn borrow_context(ptr: usize) -> ManuallyDrop<ID3D11DeviceContext> {
    ManuallyDrop::new(unsafe { std::mem::transmute::<usize, ID3D11DeviceContext>(ptr) })
}

pub(crate) unsafe fn borrow_device(ptr: usize) -> ManuallyDrop<ID3D11Device> {
    ManuallyDrop::new(unsafe { std::mem::transmute::<usize, ID3D11Device>(ptr) })
}

pub(crate) unsafe fn borrow_rtv(ptr: usize) -> ManuallyDrop<ID3D11RenderTargetView> {
    ManuallyDrop::new(unsafe { std::mem::transmute::<usize, ID3D11RenderTargetView>(ptr) })
}

// Минимальный snapshot D3D11 state, достаточный для восстановления
// после egui-directx11 рендера
pub struct D3D11StateBackup {
    render_targets: [Option<ID3D11RenderTargetView>; 1],
    depth_stencil_view: Option<ID3D11DepthStencilView>,
    blend_state: Option<ID3D11BlendState>,
    blend_factor: [f32; 4],
    blend_mask: u32,
    depth_stencil_state: Option<ID3D11DepthStencilState>,
    stencil_ref: u32,
    rasterizer_state: Option<ID3D11RasterizerState>,
    viewports: Vec<D3D11_VIEWPORT>,
    scissor_rects: Vec<windows::Win32::Foundation::RECT>,
    // Shader stages
    vs: Option<ID3D11VertexShader>,
    ps: Option<ID3D11PixelShader>,
    input_layout: Option<ID3D11InputLayout>,
    primitive_topology: D3D_PRIMITIVE_TOPOLOGY,
    // Vertex/Index buffers
    vb: [Option<ID3D11Buffer>; 1],
    vb_stride: [u32; 1],
    vb_offset: [u32; 1],
    ib: Option<ID3D11Buffer>,
    ib_format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
    ib_offset: u32,
    // PS resources
    ps_srv: [Option<ID3D11ShaderResourceView>; 1],
    ps_sampler: [Option<ID3D11SamplerState>; 1],
    // VS constant buffers
    vs_cb: [Option<ID3D11Buffer>; 1],
}

impl D3D11StateBackup {
    // Сохранить текущий pipeline state
    pub unsafe fn save(ctx: &ID3D11DeviceContext) -> Self {
        let mut render_targets: [Option<ID3D11RenderTargetView>; 1] = [None];
        let mut depth_stencil_view: Option<ID3D11DepthStencilView> = None;
        unsafe { ctx.OMGetRenderTargets(Some(&mut render_targets), Some(&mut depth_stencil_view)) };

        let mut blend_state: Option<ID3D11BlendState> = None;
        let mut blend_factor = [0.0f32; 4];
        let mut blend_mask = 0u32;
        unsafe {
            ctx.OMGetBlendState(
                Some(&mut blend_state),
                Some(&mut blend_factor),
                Some(&mut blend_mask),
            )
        };

        let mut depth_stencil_state: Option<ID3D11DepthStencilState> = None;
        let mut stencil_ref = 0u32;
        unsafe { ctx.OMGetDepthStencilState(Some(&mut depth_stencil_state), Some(&mut stencil_ref)) };

        let rasterizer_state = unsafe { ctx.RSGetState().ok() };

        let mut num_vp = 0u32;
        unsafe { ctx.RSGetViewports(&mut num_vp, None) };
        let mut viewports = vec![D3D11_VIEWPORT::default(); num_vp as usize];
        if num_vp > 0 {
            unsafe { ctx.RSGetViewports(&mut num_vp, Some(viewports.as_mut_ptr())) };
        }

        let mut num_sr = 0u32;
        unsafe { ctx.RSGetScissorRects(&mut num_sr, None) };
        let mut scissor_rects =
            vec![windows::Win32::Foundation::RECT::default(); num_sr as usize];
        if num_sr > 0 {
            unsafe { ctx.RSGetScissorRects(&mut num_sr, Some(scissor_rects.as_mut_ptr())) };
        }

        let mut vs: Option<ID3D11VertexShader> = None;
        unsafe { ctx.VSGetShader(&mut vs, None, None) };

        let mut ps: Option<ID3D11PixelShader> = None;
        unsafe { ctx.PSGetShader(&mut ps, None, None) };

        let input_layout = unsafe { ctx.IAGetInputLayout().ok() };

        let primitive_topology = unsafe { ctx.IAGetPrimitiveTopology() };

        let mut vb: [Option<ID3D11Buffer>; 1] = [None];
        let mut vb_stride = [0u32; 1];
        let mut vb_offset = [0u32; 1];
        unsafe {
            ctx.IAGetVertexBuffers(
                0,
                1,
                Some(vb.as_mut_ptr()),
                Some(vb_stride.as_mut_ptr()),
                Some(vb_offset.as_mut_ptr()),
            )
        };

        let mut ib: Option<ID3D11Buffer> = None;
        let mut ib_format = windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT::default();
        let mut ib_offset = 0u32;
        unsafe { ctx.IAGetIndexBuffer(Some(&mut ib), Some(&mut ib_format), Some(&mut ib_offset)) };

        let mut ps_srv: [Option<ID3D11ShaderResourceView>; 1] = [None];
        unsafe { ctx.PSGetShaderResources(0, Some(&mut ps_srv)) };

        let mut ps_sampler: [Option<ID3D11SamplerState>; 1] = [None];
        unsafe { ctx.PSGetSamplers(0, Some(&mut ps_sampler)) };

        let mut vs_cb: [Option<ID3D11Buffer>; 1] = [None];
        unsafe { ctx.VSGetConstantBuffers(0, Some(&mut vs_cb)) };

        Self {
            render_targets,
            depth_stencil_view,
            blend_state,
            blend_factor,
            blend_mask,
            depth_stencil_state,
            stencil_ref,
            rasterizer_state,
            viewports,
            scissor_rects,
            vs,
            ps,
            input_layout,
            primitive_topology,
            vb,
            vb_stride,
            vb_offset,
            ib,
            ib_format,
            ib_offset,
            ps_srv,
            ps_sampler,
            vs_cb,
        }
    }

    // Восстановить pipeline state
    pub unsafe fn restore(self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.OMSetRenderTargets(Some(&self.render_targets), self.depth_stencil_view.as_ref());
            ctx.OMSetBlendState(
                self.blend_state.as_ref(),
                Some(&self.blend_factor),
                self.blend_mask,
            );
            ctx.OMSetDepthStencilState(self.depth_stencil_state.as_ref(), self.stencil_ref);
            ctx.RSSetState(self.rasterizer_state.as_ref());

            if !self.viewports.is_empty() {
                ctx.RSSetViewports(Some(&self.viewports));
            }
            if !self.scissor_rects.is_empty() {
                ctx.RSSetScissorRects(Some(&self.scissor_rects));
            }

            ctx.VSSetShader(self.vs.as_ref(), None);
            ctx.PSSetShader(self.ps.as_ref(), None);
            ctx.IASetInputLayout(self.input_layout.as_ref());
            ctx.IASetPrimitiveTopology(self.primitive_topology);

            ctx.IASetVertexBuffers(
                0,
                1,
                Some(self.vb.as_ptr()),
                Some(self.vb_stride.as_ptr()),
                Some(self.vb_offset.as_ptr()),
            );
            ctx.IASetIndexBuffer(self.ib.as_ref(), self.ib_format, self.ib_offset);

            ctx.PSSetShaderResources(0, Some(&self.ps_srv));
            ctx.PSSetSamplers(0, Some(&self.ps_sampler));
            ctx.VSSetConstantBuffers(0, Some(&self.vs_cb));
        }
    }
}