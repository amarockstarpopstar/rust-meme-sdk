#[cfg(target_os = "windows")]
use crate::error::EngineError;
#[cfg(target_os = "windows")]
use crate::renderer::RenderFrame;
#[cfg(target_os = "windows")]
use glam::Vec4;
#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
    ID3D11Texture2D, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dxgi::{
    IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD,
};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_SAMPLE_DESC,
};

#[cfg(target_os = "windows")]
pub struct Dx11Renderer {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain,
    render_target: ID3D11RenderTargetView,
    width: u32,
    height: u32,
}

#[cfg(target_os = "windows")]
impl Dx11Renderer {
    pub fn new(window: &winit::window::Window) -> Result<Self, EngineError> {
        let hwnd = window_handle(window)?;
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Width: width,
                Height: height,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                ..Default::default()
            },
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: windows::Win32::Graphics::Dxgi::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            OutputWindow: hwnd,
            Windowed: true.into(),
            SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
            ..Default::default()
        };

        let mut device = None;
        let mut context = None;
        let mut swap_chain = None;

        unsafe {
            D3D11CreateDeviceAndSwapChain(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&[D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION,
                Some(&swap_chain_desc),
                Some(&mut swap_chain),
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .map_err(|err| EngineError::RendererInit(format!("D3D11 init failed: {err:?}")))?;
        }

        let device = device.ok_or_else(|| {
            EngineError::RendererInit("missing D3D11 device".to_string())
        })?;
        let context = context.ok_or_else(|| {
            EngineError::RendererInit("missing D3D11 context".to_string())
        })?;
        let swap_chain = swap_chain.ok_or_else(|| {
            EngineError::RendererInit("missing swap chain".to_string())
        })?;

        let render_target = create_render_target(&device, &swap_chain)?;
        set_viewport(&context, width, height);

        Ok(Self {
            device,
            context,
            swap_chain,
            render_target,
            width,
            height,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if width == self.width && height == self.height {
            return;
        }
        self.width = width;
        self.height = height;
        unsafe {
            self.context
                .OMSetRenderTargets(None, None);
            let _ = self
                .swap_chain
                .ResizeBuffers(0, width, height, DXGI_FORMAT_R8G8B8A8_UNORM, 0);
        }
        if let Ok(render_target) = create_render_target(&self.device, &self.swap_chain) {
            self.render_target = render_target;
            set_viewport(&self.context, width, height);
        }
    }

    pub fn render(&mut self, frame: RenderFrame) -> Result<(), EngineError> {
        let color = vec4_to_color(frame.clear_color);
        unsafe {
            self.context.OMSetRenderTargets(Some(&[Some(
                self.render_target.clone(),
            )]), None);
            self.context.ClearRenderTargetView(&self.render_target, &color);
            self.swap_chain
                .Present(1, 0)
                .ok()
                .map_err(|err| EngineError::Runtime(format!("present failed: {err:?}")))?;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn window_handle(window: &winit::window::Window) -> Result<HWND, EngineError> {
    let handle = window
        .window_handle()
        .map_err(|err| EngineError::WindowCreation(format!("window handle: {err:?}")))?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => {
            let hwnd = handle.hwnd.get() as isize;
            Ok(HWND(hwnd))
        }
        _ => Err(EngineError::UnsupportedPlatform(
            "non-Windows window handle".to_string(),
        )),
    }
}

#[cfg(target_os = "windows")]
fn create_render_target(
    device: &ID3D11Device,
    swap_chain: &IDXGISwapChain,
) -> Result<ID3D11RenderTargetView, EngineError> {
    unsafe {
        let back_buffer: ID3D11Texture2D = swap_chain
            .GetBuffer(0)
            .map_err(|err| EngineError::RendererInit(format!("back buffer: {err:?}")))?;
        let mut render_target = None;
        device
            .CreateRenderTargetView(&back_buffer, None, Some(&mut render_target))
            .map_err(|err| EngineError::RendererInit(format!("rtv: {err:?}")))?;
        render_target.ok_or_else(|| {
            EngineError::RendererInit("missing render target".to_string())
        })
    }
}

#[cfg(target_os = "windows")]
fn set_viewport(context: &ID3D11DeviceContext, width: u32, height: u32) {
    let viewport = windows::Win32::Graphics::Direct3D11::D3D11_VIEWPORT {
        Width: width as f32,
        Height: height as f32,
        MaxDepth: 1.0,
        ..Default::default()
    };
    unsafe {
        context.RSSetViewports(Some(&[viewport]));
    }
}

#[cfg(target_os = "windows")]
fn vec4_to_color(color: Vec4) -> [f32; 4] {
    [color.x, color.y, color.z, color.w]
}
