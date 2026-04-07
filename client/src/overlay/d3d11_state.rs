//! Backup / restore D3D11 pipeline state вокруг egui рендера.

use std::mem::ManuallyDrop;
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY;
use windows::Win32::Graphics::Direct3D11::*;

pub unsafe fn borrow_context(ptr: usize) -> ManuallyDrop<ID3D11DeviceContext> {
    ManuallyDrop::new(unsafe { std::mem::transmute::<usize, ID3D11DeviceContext>(ptr) })
}

pub unsafe fn borrow_device(ptr: usize) -> ManuallyDrop<ID3D11Device> {
    ManuallyDrop::new(unsafe { std::mem::transmute::<usize, ID3D11Device>(ptr) })
}

pub unsafe fn borrow_rtv(ptr: usize) -> ManuallyDrop<ID3D11RenderTargetView> {
    ManuallyDrop::new(unsafe { std::mem::transmute::<usize, ID3D11RenderTargetView>(ptr) })
}

pub struct StateBackup {
    render_targets: [Option<ID3D11RenderTargetView>; 1],
    dsv: Option<ID3D11DepthStencilView>,
    blend: Option<ID3D11BlendState>,
    blend_factor: [f32; 4],
    blend_mask: u32,
    ds_state: Option<ID3D11DepthStencilState>,
    stencil_ref: u32,
    rasterizer: Option<ID3D11RasterizerState>,
    viewports: Vec<D3D11_VIEWPORT>,
    scissors: Vec<windows::Win32::Foundation::RECT>,
    vs: Option<ID3D11VertexShader>,
    ps: Option<ID3D11PixelShader>,
    input_layout: Option<ID3D11InputLayout>,
    topology: D3D_PRIMITIVE_TOPOLOGY,
    vb: [Option<ID3D11Buffer>; 1],
    vb_stride: [u32; 1],
    vb_offset: [u32; 1],
    ib: Option<ID3D11Buffer>,
    ib_format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
    ib_offset: u32,
    ps_srv: [Option<ID3D11ShaderResourceView>; 1],
    ps_sampler: [Option<ID3D11SamplerState>; 1],
    vs_cb: [Option<ID3D11Buffer>; 1],
}

impl StateBackup {
    pub unsafe fn save(ctx: &ID3D11DeviceContext) -> Self {
        unsafe {
            let mut render_targets: [Option<ID3D11RenderTargetView>; 1] = [None];
            let mut dsv: Option<ID3D11DepthStencilView> = None;
            ctx.OMGetRenderTargets(Some(&mut render_targets), Some(&mut dsv));

            let mut blend: Option<ID3D11BlendState> = None;
            let mut blend_factor = [0.0f32; 4];
            let mut blend_mask = 0u32;
            ctx.OMGetBlendState(Some(&mut blend), Some(&mut blend_factor), Some(&mut blend_mask));

            let mut ds_state: Option<ID3D11DepthStencilState> = None;
            let mut stencil_ref = 0u32;
            ctx.OMGetDepthStencilState(Some(&mut ds_state), Some(&mut stencil_ref));

            let rasterizer = ctx.RSGetState().ok();

            let mut num_vp = 0u32;
            ctx.RSGetViewports(&mut num_vp, None);
            let mut viewports = vec![D3D11_VIEWPORT::default(); num_vp as usize];
            if num_vp > 0 {
                ctx.RSGetViewports(&mut num_vp, Some(viewports.as_mut_ptr()));
            }

            let mut num_sr = 0u32;
            ctx.RSGetScissorRects(&mut num_sr, None);
            let mut scissors = vec![windows::Win32::Foundation::RECT::default(); num_sr as usize];
            if num_sr > 0 {
                ctx.RSGetScissorRects(&mut num_sr, Some(scissors.as_mut_ptr()));
            }

            let mut vs: Option<ID3D11VertexShader> = None;
            ctx.VSGetShader(&mut vs, None, None);

            let mut ps: Option<ID3D11PixelShader> = None;
            ctx.PSGetShader(&mut ps, None, None);

            let input_layout = ctx.IAGetInputLayout().ok();
            let topology = ctx.IAGetPrimitiveTopology();

            let mut vb: [Option<ID3D11Buffer>; 1] = [None];
            let mut vb_stride = [0u32; 1];
            let mut vb_offset = [0u32; 1];
            ctx.IAGetVertexBuffers(0, 1, Some(vb.as_mut_ptr()), Some(vb_stride.as_mut_ptr()), Some(vb_offset.as_mut_ptr()));

            let mut ib: Option<ID3D11Buffer> = None;
            let mut ib_format = windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT::default();
            let mut ib_offset = 0u32;
            ctx.IAGetIndexBuffer(Some(&mut ib), Some(&mut ib_format), Some(&mut ib_offset));

            let mut ps_srv: [Option<ID3D11ShaderResourceView>; 1] = [None];
            ctx.PSGetShaderResources(0, Some(&mut ps_srv));

            let mut ps_sampler: [Option<ID3D11SamplerState>; 1] = [None];
            ctx.PSGetSamplers(0, Some(&mut ps_sampler));

            let mut vs_cb: [Option<ID3D11Buffer>; 1] = [None];
            ctx.VSGetConstantBuffers(0, Some(&mut vs_cb));

            Self {
                render_targets, dsv, blend, blend_factor, blend_mask,
                ds_state, stencil_ref, rasterizer, viewports, scissors,
                vs, ps, input_layout, topology, vb, vb_stride, vb_offset,
                ib, ib_format, ib_offset, ps_srv, ps_sampler, vs_cb,
            }
        }
    }

    pub unsafe fn restore(self, ctx: &ID3D11DeviceContext) {
        unsafe {
            ctx.OMSetRenderTargets(Some(&self.render_targets), self.dsv.as_ref());
            ctx.OMSetBlendState(self.blend.as_ref(), Some(&self.blend_factor), self.blend_mask);
            ctx.OMSetDepthStencilState(self.ds_state.as_ref(), self.stencil_ref);
            ctx.RSSetState(self.rasterizer.as_ref());
            if !self.viewports.is_empty() { ctx.RSSetViewports(Some(&self.viewports)); }
            if !self.scissors.is_empty() { ctx.RSSetScissorRects(Some(&self.scissors)); }
            ctx.VSSetShader(self.vs.as_ref(), None);
            ctx.PSSetShader(self.ps.as_ref(), None);
            ctx.IASetInputLayout(self.input_layout.as_ref());
            ctx.IASetPrimitiveTopology(self.topology);
            ctx.IASetVertexBuffers(0, 1, Some(self.vb.as_ptr()), Some(self.vb_stride.as_ptr()), Some(self.vb_offset.as_ptr()));
            ctx.IASetIndexBuffer(self.ib.as_ref(), self.ib_format, self.ib_offset);
            ctx.PSSetShaderResources(0, Some(&self.ps_srv));
            ctx.PSSetSamplers(0, Some(&self.ps_sampler));
            ctx.VSSetConstantBuffers(0, Some(&self.vs_cb));
        }
    }
}