use macroquad::prelude::*;

#[macroquad::main("Lighting Example")]
async fn main() {
    let render_target = render_target(screen_width() as u32, screen_height() as u32);
    render_target.texture.set_filter(FilterMode::Linear);

    let material = load_material(
        ShaderSource::Glsl { 
            vertex: VERTEX_SHADER, 
            fragment: FRAGMENT_SHADER 
        },
        MaterialParams { 
            uniforms: vec![
                UniformDesc::new("Darkness", UniformType::Float1),
                UniformDesc::new("LightPos", UniformType::Float2),
                UniformDesc::new("screenWidth", UniformType::Float1),
                UniformDesc::new("screenHeight", UniformType::Float1),
            ],
            textures: vec!["tex".to_string()],
            ..Default::default() 
        }
    ).unwrap();

    loop {
        set_camera(&Camera2D {
            render_target: Some(render_target.clone()),
            ..Default::default()
        });

        clear_background(LIGHTGRAY);

        draw_line(-0.4, 0.4, -0.8, 0.9, 0.05, BLUE);
        draw_rectangle(-0.3, 0.3, 0.2, 0.2, GREEN);
        draw_circle(0., 0., 0.1, YELLOW);

        set_default_camera();

        material.set_texture("tex", render_target.texture.clone());

        let (mx, my) = mouse_position();

        material.set_uniform("Darkness", 0.8f32);
        material.set_uniform("LightPos", vec2(mx, my));
        material.set_uniform("screenWidth", screen_width());
        material.set_uniform("screenHeight", screen_height());

        gl_use_material(&material);
        draw_rectangle(0., 0., screen_width(), screen_height(), WHITE);
        gl_use_default_material();

        next_frame().await
    }
}

const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 100
precision mediump float;
varying vec2 uv;
uniform float Darkness;          
uniform sampler2D tex;          
uniform vec2 LightPos;          
uniform float screenWidth;          
uniform float screenHeight;          

void main() {
    vec4 base = texture2D(tex, uv);

    vec2 fragPos = uv * vec2(screenWidth, screenHeight);

    float dist = distance(fragPos, LightPos);

    float radius = 150.0;
    float softness = 80.0;
    float light = 1.0 - smoothstep(radius, radius + softness, dist);

    vec3 darkened = base.rgb * (1.0 - Darkness * (1.0 - light));

    gl_FragColor = vec4(darkened, base.a);
}
"#;