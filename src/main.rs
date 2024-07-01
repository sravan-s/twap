use std::{sync::{Arc, Mutex}, vec};

use glium::{implement_vertex, uniform, Surface};
use sysinfo::System;
use tokio::time::sleep;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
}

fn make_bar(idx: u16, width:f32) -> [Vertex; 4] {
    let x0: f32 = ((1 + idx) as f32 * width) - 1.0;
    let x1: f32 = ((2 + idx) as f32 * width) - 1.0;
    let y0 = -0.7;
    let y1 = -0.8;
    [
        Vertex { position: [x0,  y0] },
        Vertex { position: [x1,  y0] },
        Vertex { position: [x0, y1] },
        Vertex { position: [x1, y1] },
    ]
}

fn make_bars(len: u16) -> Vec<[Vertex; 4]> {
    let mut bars = vec![];
    let width = 2.0 / ((len + 1) * 2) as f32;
    for i in 0..=len {
        bars.push(make_bar(i * 2, width));
    }
    bars
}

#[tokio::main]
async fn main() {
    let mut sys = System::new_all();

    // kept it here to avoid cloning sys in the tokio::spawn block
    let shapes = make_bars(sys.cpus().len() as u16);
    // println!("shape: {:?}", shapes);
    let cpu_usage = Arc::new(Mutex::new(vec![0.0]));
    let cpu_usage_clone = Arc::clone(&cpu_usage);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            sys.refresh_cpu(); // Refreshing CPU information.
            let mut usage = vec![];
            for cpu in sys.cpus() {
                usage.push(cpu.cpu_usage());
            }
            // usage /= sys.cpus().len() as f32;
            *cpu_usage.lock().unwrap() = usage;
            // Sleeping to let time for the system to run for long
            // enough to have useful information.
            sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
        }
    });

    let event_loop = winit::event_loop::EventLoopBuilder::new().build().expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().with_inner_size(800, 600).build(&event_loop);

    implement_vertex!(Vertex, position);
    
    let vertex_buffers = shapes.iter().map(|shape| {
        glium::VertexBuffer::new(&display, shape).unwrap()
    }).collect::<Vec<_>>();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        uniform float y_off;

        void main() {
            vec2 pos = position;
            if (pos.y > -0.8) {
                pos.y = (y_off / 100.0) - 0.8;
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
                    let cpu_usages = (*cpu_usage_clone.lock().unwrap()).clone();
                    // let y_offs = cpu_usages.iter().map(|usage| (usage / 100.0) - 1.0).collect::<Vec<f32>>();
                    // println!("\n{}% ", y_offs);

                    let mut target = display.draw();
                    target.clear_color(0.0, 0.0, 1.0, 1.0);

                    vertex_buffers.iter().zip(cpu_usages.iter()).for_each(|(vertex_buffer, y_off)| {
                        let uniform = uniform! { y_off: *y_off };
                        target.draw(vertex_buffer, &indices, &program, &uniform, &Default::default()).unwrap();
                    });

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
