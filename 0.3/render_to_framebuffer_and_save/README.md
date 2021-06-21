# Render to Framebuffer and Save as Image
The purpose of this example is, to render a GUI to an off-screen framebuffer and save it to an image. So, it is possible to create "screenshots" of an app without actually opening a window of the OS.

It is based on [this](https://github.com/hecrj/iced/tree/master/examples/integration) Iced example in combination with [this](https://github.com/gfx-rs/wgpu-rs/tree/master/examples/capture) wgpu example.

![Screenshot of GUI](https://user-images.githubusercontent.com/12140954/122641248-5b7e2480-d104-11eb-9852-ef621ebe9a71.png)

## What this does
It simply renders an arbitrary Iced app (in this case a simple counter app) to a framebuffer. Afterwards the buffer is saved to the disk as a png.

## How it works

1. A wgpu adapter as well as an output buffer are created
```rust
    let adapter = wgpu::Instance::new(wgpu::BackendBit::PRIMARY)
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();

    let (mut device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default()
            },
            None,
        )
        .await
        .unwrap();

    // It is a WebGPU requirement that ImageCopyBuffer.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
    // So we calculate padded_bytes_per_row by rounding unpadded_bytes_per_row
    // up to the next multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
    // https://en.wikipedia.org/wiki/Data_structure_alignment#Computing_padding
    let buffer_dimensions = BufferDimensions::new(width, height);
    // The output buffer lets us retrieve the data as an array
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
        usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });
```

2. A texture is created, on which the GUI will be drawn. Also some other objects like the encoder are created which are needed by the renderer.
```rust

    let texture_extent = wgpu::Extent3d {
        width: buffer_dimensions.width as u32,
        height: buffer_dimensions.height as u32,
        depth: 1
    };

    // The render pipeline renders data into this texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
        label: None,
    });

    let mut debug = Debug::new();
    let mut renderer = Renderer::new(Backend::new(&mut device, Settings::default()));

    // Initialize staging belt
    let mut staging_belt = wgpu::util::StagingBelt::new(5 * 1024);

    let viewport = Viewport::with_physical_size(
        Size::new(width as u32, height as u32),
        1.0_f64,
    );
    let cursor_position = PhysicalPosition::new(1.0, 1.0);

    let state = program::State::new(
        appl,
        viewport.logical_size(),
        conversion::cursor_position(cursor_position, viewport.scale_factor()),
        &mut renderer,
        &mut debug,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    renderer.backend_mut().draw(
        &mut device,
        &mut staging_belt,
        &mut encoder,
        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
        &viewport,
        state.primitive(),
        &debug.overlay(),
    );
```

3. It takes the output buffer and writes it to the disk as a PNG
```rust
    // Note that we're not calling `.await` here.
    let buffer_slice = output_buffer.slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    device.poll(wgpu::Maintain::Wait);
    // If a file system is available, write the buffer as a PNG
    let has_file_system_available = cfg!(not(target_arch = "wasm32"));
    if !has_file_system_available {
        return;
    }

    if let Ok(()) = buffer_future.await {
        let padded_buffer = buffer_slice.get_mapped_range();

        let mut png_encoder = png::Encoder::new(
            File::create(png_output_path).unwrap(),
            buffer_dimensions.width as u32,
            buffer_dimensions.height as u32,
        );
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(png::ColorType::RGBA);
        let mut png_writer = png_encoder
            .write_header()
            .unwrap()
            .into_stream_writer_with_size(buffer_dimensions.unpadded_bytes_per_row);

        // from the padded_buffer we write just the unpadded bytes into the image
        for chunk in padded_buffer.chunks(buffer_dimensions.padded_bytes_per_row) {
            png_writer
                .write_all(&chunk[..buffer_dimensions.unpadded_bytes_per_row])
                .unwrap();
        }
        png_writer.finish().unwrap();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(padded_buffer);

        output_buffer.unmap();
```
