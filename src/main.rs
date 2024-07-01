use std::sync::{Arc, Mutex};

use glium::{implement_vertex, uniform, Surface};
use sysinfo::System;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let mut sys = System::new_all();
    let cpu_usage = Arc::new(Mutex::new(0.0));
    let cpu_usage_clone = Arc::clone(&cpu_usage);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            sys.refresh_cpu(); // Refreshing CPU information.
            let mut usage = 0.0;
            for cpu in sys.cpus() {
                usage += cpu.cpu_usage();
            }
            // usage /= sys.cpus().len() as f32;
            *cpu_usage.lock().unwrap() = usage;
            // Sleeping to let time for the system to run for long
            // enough to have useful information.
            sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
        }
    });

    let event_loop = winit::event_loop::EventLoopBuilder::new().build().expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().with_inner_size(1600, 1600).build(&event_loop);

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
    implement_vertex!(Vertex, position);
    let shape = vec![
        Vertex { position: [-0.5,  1.0] },
        Vertex { position: [ 0.5,  1.0] },
        Vertex { position: [-0.5, -1.0] },
        Vertex { position: [ 0.5, -1.0] },
    ];
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        uniform float y_off;

        void main() {
            vec2 pos = position;
            if (pos.y > -1.0) {
                pos.y = y_off;
            }
            gl_Position = vec4(pos, 0.0, 1.0);
        }
    "#;
    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    event_loop.run(move |ev, window_target| {
        match ev {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    window_target.exit();
                },
                // We now need to render everyting in response to a RedrawRequested event due to the animation
                winit::event::WindowEvent::RedrawRequested => {
                    let y_off = (*cpu_usage_clone.lock().unwrap() / 100.0) - 1.0;
                    print!("\n{}% ", y_off);

                    let mut target = display.draw();
                    target.clear_color(0.0, 0.0, 1.0, 1.0);
                    let uniforms = uniform! { y_off: y_off };
                    target.draw(&vertex_buffer, &indices, &program, &uniforms,
                                &Default::default()).unwrap();
                    target.finish().unwrap();
                },
                // Because glium doesn't know about windows we need to resize the display
                // when the window's size has changed.
                winit::event::WindowEvent::Resized(window_size) => {
                    display.resize(window_size.into());
                },
                _ => (),
            },
            // By requesting a redraw in response to a RedrawEventsCleared event we get continuous rendering.
            // For applications that only change due to user input you could remove this handler.
            winit::event::Event::AboutToWait => {
                window.request_redraw();
            },
            _ => (),
        }
    })
    .unwrap();
}
