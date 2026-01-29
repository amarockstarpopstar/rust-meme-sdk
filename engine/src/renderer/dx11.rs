#[cfg(target_os = "windows")]
use crate::error::EngineError;
#[cfg(target_os = "windows")]
use crate::renderer::RenderFrame;
#[cfg(target_os = "windows")]
use glam::{Mat4, Vec3, Vec4};
#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use std::ffi::CString;
#[cfg(target_os = "windows")]
use std::mem::size_of;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0, D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Direct3D::Fxc::{
    D3DCompile, D3DCOMPILE_ENABLE_STRICTNESS, D3DCOMPILE_OPTIMIZATION_LEVEL3,
};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDeviceAndSwapChain, D3D11_BUFFER_DESC, D3D11_INPUT_ELEMENT_DESC,
    D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC, ID3D11Buffer, ID3D11DepthStencilView,
    ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader,
    ID3D11RenderTargetView, ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER,
    D3D11_BIND_DEPTH_STENCIL, D3D11_BIND_INDEX_BUFFER, D3D11_BIND_VERTEX_BUFFER,
    D3D11_CLEAR_DEPTH, D3D11_CLEAR_STENCIL, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_INPUT_PER_VERTEX_DATA, D3D11_SDK_VERSION, D3D11_USAGE_DEFAULT,
};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dxgi::{
    IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD,
};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_D24_UNORM_S8_UINT, DXGI_FORMAT_R16_UINT, DXGI_FORMAT_R32G32B32_FLOAT,
    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_SAMPLE_DESC,
};
#[cfg(target_os = "windows")]
use windows::core::PCSTR;

#[cfg(target_os = "windows")]
pub struct Dx11Renderer {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain,
    render_target: ID3D11RenderTargetView,
    depth_view: ID3D11DepthStencilView,
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    input_layout: ID3D11InputLayout,
    vertex_buffer: ID3D11Buffer,
    index_buffer: ID3D11Buffer,
    constant_buffer: ID3D11Buffer,
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
        let depth_view = create_depth_stencil_view(&device, width, height)?;
        set_viewport(&context, width, height);
        let shader_bundle = create_shaders(&device)?;
        let buffers = create_cube_buffers(&device)?;

        Ok(Self {
            device,
            context,
            swap_chain,
            render_target,
            depth_view,
            vertex_shader: shader_bundle.vertex_shader,
            pixel_shader: shader_bundle.pixel_shader,
            input_layout: shader_bundle.input_layout,
            vertex_buffer: buffers.vertex_buffer,
            index_buffer: buffers.index_buffer,
            constant_buffer: buffers.constant_buffer,
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
        if let Ok(depth_view) = create_depth_stencil_view(&self.device, width, height) {
            self.depth_view = depth_view;
        }
    }

    pub fn render(&mut self, frame: RenderFrame) -> Result<(), EngineError> {
        let color = vec4_to_color(frame.clear_color);
        let transform = cube_transform(frame.time_seconds, self.width, self.height);
        let constant_data = ConstantBuffer {
            mvp: transform.to_cols_array_2d(),
        };
        unsafe {
            self.context.OMSetRenderTargets(
                Some(&[Some(self.render_target.clone())]),
                Some(&self.depth_view),
            );
            self.context.ClearRenderTargetView(&self.render_target, &color);
            self.context.ClearDepthStencilView(
                &self.depth_view,
                (D3D11_CLEAR_DEPTH | D3D11_CLEAR_STENCIL).0,
                1.0,
                0,
            );
            self.context.IASetInputLayout(Some(&self.input_layout));
            let stride = size_of::<Vertex>() as u32;
            let offset = 0u32;
            let buffers = [Some(self.vertex_buffer.clone())];
            let strides = [stride];
            let offsets = [offset];
            self.context.IASetVertexBuffers(
                0,
                1,
                Some(buffers.as_ptr()),
                Some(strides.as_ptr()),
                Some(offsets.as_ptr()),
            );
            self.context.IASetIndexBuffer(&self.index_buffer, DXGI_FORMAT_R16_UINT, 0);
            self.context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.context.VSSetShader(Some(&self.vertex_shader), None);
            self.context.PSSetShader(Some(&self.pixel_shader), None);
            self.context.UpdateSubresource(
                &self.constant_buffer,
                0,
                None,
                &constant_data as *const ConstantBuffer as *const _,
                0,
                0,
            );
            self.context
                .VSSetConstantBuffers(0, Some(&[Some(self.constant_buffer.clone())]));
            self.context.DrawIndexed(CUBE_INDEX_COUNT, 0, 0);
            self.swap_chain
                .Present(1, 0)
                .ok()
                .map_err(|err| EngineError::Runtime(format!("present failed: {err:?}")))?;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Copy, Clone)]
struct ConstantBuffer {
    mvp: [[f32; 4]; 4],
}

#[cfg(target_os = "windows")]
struct ShaderBundle {
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    input_layout: ID3D11InputLayout,
}

#[cfg(target_os = "windows")]
struct CubeBuffers {
    vertex_buffer: ID3D11Buffer,
    index_buffer: ID3D11Buffer,
    constant_buffer: ID3D11Buffer,
}

#[cfg(target_os = "windows")]
const CUBE_INDEX_COUNT: u32 = 36;

#[cfg(target_os = "windows")]
fn cube_vertices() -> [Vertex; 8] {
    [
        Vertex {
            position: [-1.0, -1.0, -1.0],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, -1.0, -1.0],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.0, -1.0],
            color: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-1.0, 1.0, -1.0],
            color: [1.0, 1.0, 0.0],
        },
        Vertex {
            position: [-1.0, -1.0, 1.0],
            color: [1.0, 0.0, 1.0],
        },
        Vertex {
            position: [1.0, -1.0, 1.0],
            color: [0.0, 1.0, 1.0],
        },
        Vertex {
            position: [1.0, 1.0, 1.0],
            color: [1.0, 1.0, 1.0],
        },
        Vertex {
            position: [-1.0, 1.0, 1.0],
            color: [0.1, 0.6, 0.9],
        },
    ]
}

#[cfg(target_os = "windows")]
fn cube_indices() -> [u16; CUBE_INDEX_COUNT as usize] {
    [
        0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 4, 5, 1, 4, 1, 0, 3, 2, 6, 3, 6, 7, 1,
        5, 6, 1, 6, 2, 4, 0, 3, 4, 3, 7,
    ]
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

#[cfg(target_os = "windows")]
fn create_depth_stencil_view(
    device: &ID3D11Device,
    width: u32,
    height: u32,
) -> Result<ID3D11DepthStencilView, EngineError> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_D24_UNORM_S8_UINT,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_DEPTH_STENCIL.0 as u32,
        ..Default::default()
    };
    unsafe {
        let mut depth_texture = None;
        device
            .CreateTexture2D(&desc, None, Some(&mut depth_texture))
            .map_err(|err| EngineError::RendererInit(format!("depth texture: {err:?}")))?;
        let depth_texture = depth_texture.ok_or_else(|| {
            EngineError::RendererInit("missing depth texture".to_string())
        })?;
        let mut depth_view = None;
        device
            .CreateDepthStencilView(&depth_texture, None, Some(&mut depth_view))
            .map_err(|err| EngineError::RendererInit(format!("depth view: {err:?}")))?;
        depth_view.ok_or_else(|| EngineError::RendererInit("missing depth view".to_string()))
    }
}

#[cfg(target_os = "windows")]
fn create_shaders(device: &ID3D11Device) -> Result<ShaderBundle, EngineError> {
    let vertex_source = r#"
cbuffer Frame : register(b0) {
    float4x4 mvp;
};
struct VSInput {
    float3 position : POSITION;
    float3 color : COLOR;
};
struct VSOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR;
};
VSOutput main(VSInput input) {
    VSOutput output;
    output.position = mul(mvp, float4(input.position, 1.0));
    output.color = input.color;
    return output;
}
"#;
    let pixel_source = r#"
struct PSInput {
    float4 position : SV_POSITION;
    float3 color : COLOR;
};
float4 main(PSInput input) : SV_TARGET {
    return float4(input.color, 1.0);
}
"#;
    let vertex_blob = compile_shader(vertex_source, "main", "vs_5_0")?;
    let pixel_blob = compile_shader(pixel_source, "main", "ps_5_0")?;

    unsafe {
        let mut vertex_shader = None;
        let vertex_shader_bytes = std::slice::from_raw_parts(
            vertex_blob.GetBufferPointer() as *const u8,
            vertex_blob.GetBufferSize(),
        );
        device
            .CreateVertexShader(vertex_shader_bytes, None, Some(&mut vertex_shader))
            .map_err(|err| EngineError::RendererInit(format!("vertex shader: {err:?}")))?;
        let vertex_shader = vertex_shader.ok_or_else(|| {
            EngineError::RendererInit("missing vertex shader".to_string())
        })?;

        let mut pixel_shader = None;
        let pixel_shader_bytes = std::slice::from_raw_parts(
            pixel_blob.GetBufferPointer() as *const u8,
            pixel_blob.GetBufferSize(),
        );
        device
            .CreatePixelShader(pixel_shader_bytes, None, Some(&mut pixel_shader))
            .map_err(|err| EngineError::RendererInit(format!("pixel shader: {err:?}")))?;
        let pixel_shader = pixel_shader.ok_or_else(|| {
            EngineError::RendererInit("missing pixel shader".to_string())
        })?;

        let input_elements = [
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: PCSTR(b"POSITION\0".as_ptr()),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D11_INPUT_ELEMENT_DESC {
                SemanticName: PCSTR(b"COLOR\0".as_ptr()),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 12,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];
        let mut input_layout = None;
        device
            .CreateInputLayout(&input_elements, vertex_shader_bytes, Some(&mut input_layout))
            .map_err(|err| EngineError::RendererInit(format!("input layout: {err:?}")))?;
        let input_layout = input_layout.ok_or_else(|| {
            EngineError::RendererInit("missing input layout".to_string())
        })?;

        Ok(ShaderBundle {
            vertex_shader,
            pixel_shader,
            input_layout,
        })
    }
}

#[cfg(target_os = "windows")]
fn compile_shader(
    source: &str,
    entry: &str,
    target: &str,
) -> Result<windows::Win32::Graphics::Direct3D::ID3DBlob, EngineError> {
    let mut shader_blob = None;
    let mut error_blob = None;
    let entry = CString::new(entry)
        .map_err(|err| EngineError::RendererInit(format!("shader entry: {err:?}")))?;
    let target = CString::new(target)
        .map_err(|err| EngineError::RendererInit(format!("shader target: {err:?}")))?;
    unsafe {
        D3DCompile(
            source.as_ptr() as *const _,
            source.len(),
            PCSTR::null(),
            None,
            None,
            PCSTR(entry.as_ptr().cast()),
            PCSTR(target.as_ptr().cast()),
            D3DCOMPILE_ENABLE_STRICTNESS | D3DCOMPILE_OPTIMIZATION_LEVEL3,
            0,
            &mut shader_blob,
            Some(&mut error_blob),
        )
        .map_err(|err| {
            if let Some(error_blob) = error_blob {
                let message = std::slice::from_raw_parts(
                    error_blob.GetBufferPointer() as *const u8,
                    error_blob.GetBufferSize(),
                );
                let message = String::from_utf8_lossy(message);
                EngineError::RendererInit(format!("shader compile: {message}"))
            } else {
                EngineError::RendererInit(format!("shader compile: {err:?}"))
            }
        })?;
    }
    shader_blob.ok_or_else(|| EngineError::RendererInit("missing shader blob".to_string()))
}

#[cfg(target_os = "windows")]
fn create_cube_buffers(device: &ID3D11Device) -> Result<CubeBuffers, EngineError> {
    let vertices = cube_vertices();
    let indices = cube_indices();
    let vertex_buffer_desc = D3D11_BUFFER_DESC {
        ByteWidth: (size_of::<Vertex>() * vertices.len()) as u32,
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
        ..Default::default()
    };
    let vertex_data = D3D11_SUBRESOURCE_DATA {
        pSysMem: vertices.as_ptr() as *const _,
        ..Default::default()
    };

    let index_buffer_desc = D3D11_BUFFER_DESC {
        ByteWidth: (size_of::<u16>() * indices.len()) as u32,
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_INDEX_BUFFER.0 as u32,
        ..Default::default()
    };
    let index_data = D3D11_SUBRESOURCE_DATA {
        pSysMem: indices.as_ptr() as *const _,
        ..Default::default()
    };

    let constant_buffer_desc = D3D11_BUFFER_DESC {
        ByteWidth: size_of::<ConstantBuffer>() as u32,
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
        ..Default::default()
    };

    unsafe {
        let mut vertex_buffer = None;
        device
            .CreateBuffer(&vertex_buffer_desc, Some(&vertex_data), Some(&mut vertex_buffer))
            .map_err(|err| EngineError::RendererInit(format!("vertex buffer: {err:?}")))?;
        let vertex_buffer = vertex_buffer.ok_or_else(|| {
            EngineError::RendererInit("missing vertex buffer".to_string())
        })?;

        let mut index_buffer = None;
        device
            .CreateBuffer(&index_buffer_desc, Some(&index_data), Some(&mut index_buffer))
            .map_err(|err| EngineError::RendererInit(format!("index buffer: {err:?}")))?;
        let index_buffer = index_buffer.ok_or_else(|| {
            EngineError::RendererInit("missing index buffer".to_string())
        })?;

        let mut constant_buffer = None;
        device
            .CreateBuffer(&constant_buffer_desc, None, Some(&mut constant_buffer))
            .map_err(|err| EngineError::RendererInit(format!("constant buffer: {err:?}")))?;
        let constant_buffer = constant_buffer.ok_or_else(|| {
            EngineError::RendererInit("missing constant buffer".to_string())
        })?;

        Ok(CubeBuffers {
            vertex_buffer,
            index_buffer,
            constant_buffer,
        })
    }
}

#[cfg(target_os = "windows")]
fn cube_transform(time_seconds: f32, width: u32, height: u32) -> Mat4 {
    let aspect = width as f32 / height.max(1) as f32;
    let projection = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
    let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, -5.0), Vec3::ZERO, Vec3::Y);
    let rotation =
        Mat4::from_rotation_y(time_seconds * 0.8) * Mat4::from_rotation_x(time_seconds * 0.6);
    projection * view * rotation
}
