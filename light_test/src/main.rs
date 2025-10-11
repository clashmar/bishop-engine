use macroquad::prelude::*;

#[macroquad::main("Lighting Example")]
async fn main() {
    let scene_rt = render_target(screen_width() as u32, screen_height() as u32);
    let ambient_rt = render_target(screen_width() as u32, screen_height() as u32);
    let spot_rt = render_target(screen_width() as u32, screen_height() as u32);
    let glow_rt = render_target(screen_width() as u32, screen_height() as u32);
    for rt in [&scene_rt, &ambient_rt, &spot_rt, &glow_rt] {
        rt.texture.set_filter(FilterMode::Nearest);
    }

    let cat_tex = load_texture("/Users/charliesovn/Projects/Aseprite/cat.png").await.unwrap();
    cat_tex.set_filter(FilterMode::Nearest);

    let ambient_material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: AMB_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("Darkness", UniformType::Float1),
            ],
            textures: vec!["tex".to_string()],
            ..Default::default() 
        },
    ).unwrap();

    let spot_material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: SPOT_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("Brightness", UniformType::Float1),
                UniformDesc::new("Color", UniformType::Float3),
                UniformDesc::new("ColorIntensity", UniformType::Float1),
                UniformDesc::new("Radius", UniformType::Float1),
                UniformDesc::new("Spread", UniformType::Float1),
                UniformDesc::new("LightPos", UniformType::Float2),
                UniformDesc::new("screenWidth", UniformType::Float1),
                UniformDesc::new("screenHeight", UniformType::Float1),
                UniformDesc::new("Darkness", UniformType::Float1),
                UniformDesc::new("SpotAlpha", UniformType::Float1),
            ],
            textures: vec!["tex".to_string()],
            ..Default::default() 
        },
    ).unwrap();

    
    let glow_material = load_material(
        ShaderSource::Glsl { 
            vertex: VERTEX_SHADER, 
            fragment: GLOW_FRAGMENT_SHADER 
        },
        MaterialParams { 
            uniforms: vec![
                UniformDesc::new("Brightness", UniformType::Float1),
                UniformDesc::new("Color", UniformType::Float3),
                UniformDesc::new("ColorIntensity", UniformType::Float1),
                UniformDesc::new("LightPos", UniformType::Float2),
                UniformDesc::new("Glow", UniformType::Float1),
                UniformDesc::new("maskWidth", UniformType::Float1),
                UniformDesc::new("maskHeight", UniformType::Float1),
                UniformDesc::new("maskPos", UniformType::Float2),
                UniformDesc::new("maskSize", UniformType::Float2),
                UniformDesc::new("screenWidth", UniformType::Float1),
                UniformDesc::new("screenHeight", UniformType::Float1),
                UniformDesc::new("Darkness", UniformType::Float1),
            ],
            textures: vec!["scene_tex".to_string(), "tex_mask".to_string()],
            ..Default::default() 
        }
    ).unwrap();

    let composite_material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: COMPOSITE_FRAGMENT_SHADER,
        },
        MaterialParams {
            textures: vec![
                "ambient_tex".to_string(),
                "spot_tex".to_string(),
                "glow_tex".to_string(),
            ],
            ..Default::default()
        },
    ).unwrap();

    loop {
        let (mx, my) = mouse_position();

        // Draw scene
        let pixel_cam = Camera2D {
            target: vec2(screen_width() * 0.5, screen_height() * 0.5),
            zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
            render_target: Some(scene_rt.clone()),
            ..Default::default()
        };
        set_camera(&pixel_cam);
        clear_background(DARKBLUE);

        draw_line(100.0, 200.0,  50.0, 300.0, 5.0, BLUE);
        draw_rectangle(300.0, 150.0, 120.0, 80.0, GREEN);
        draw_circle(500.0, 250.0, 60.0, YELLOW);

        let cat_w = 0.2 * screen_width();
        let cat_h = 0.2 * screen_height(); 
        let cat_x = 200.0;
        let cat_y = 200.0;
        let cat_center = vec2(cat_x + cat_w * 0.5, cat_y + cat_h * 0.5);

        draw_texture_ex(
            &cat_tex,
            cat_x,
            cat_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(cat_w, cat_h)),
                ..Default::default()
            },
        );

        set_default_camera();

        // Constants
        let darkness = 0.8f32;
        let brightness = 0.9f32;
        let spot_alpha = 0.6f32;
        let light_color_intensity = 0.0f32;
        let light_color = vec3(1.0, 0.85, 0.6);
        let radius = 20.0f32;
        let spread = 150.0f32;
        let glow = 1.0f32;
        
        {
            // Ambient pass
            ambient_material.set_texture("tex", scene_rt.texture.clone());
            ambient_material.set_uniform("Darkness", darkness);

            let cam = Camera2D {
                target: vec2(screen_width() * 0.5, screen_height() * 0.5),
                zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
                render_target: Some(ambient_rt.clone()),
                ..Default::default()
            };
            set_camera(&cam);

            gl_use_material(&ambient_material);
            draw_texture_ex(
                &scene_rt.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
            gl_use_default_material();
            set_default_camera();
        }
        
        {
            // Spotlight pass
            spot_material.set_texture("tex", scene_rt.texture.clone());
            spot_material.set_uniform("ColorIntensity", light_color_intensity);
            spot_material.set_uniform("Brightness", brightness);
            spot_material.set_uniform("Color", light_color);
            spot_material.set_uniform("Radius", radius);        
            spot_material.set_uniform("Spread", spread); 
            spot_material.set_uniform("LightPos", vec2(mx, my));
            spot_material.set_uniform("screenWidth", screen_width());
            spot_material.set_uniform("screenHeight", screen_height());
            spot_material.set_uniform("Darkness", darkness);
            spot_material.set_uniform("SpotAlpha", spot_alpha);

            let cam = Camera2D {
                target: vec2(screen_width() * 0.5, screen_height() * 0.5),
                zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
                render_target: Some(spot_rt.clone()),
                ..Default::default()
            };
            set_camera(&cam);

            gl_use_material(&spot_material);
            draw_texture_ex(
                &scene_rt.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
            gl_use_default_material();
            set_default_camera();
        }
        
        {
            // Glow pass
            glow_material.set_texture("scene_tex", scene_rt.texture.clone());
            glow_material.set_texture("tex_mask", cat_tex.clone());
            glow_material.set_uniform("ColorIntensity", 0.4f32);
            glow_material.set_uniform("Brightness", 0.4f32);
            glow_material.set_uniform("Color", light_color);
            glow_material.set_uniform("LightPos", vec2(mx, my));
            glow_material.set_uniform("screenWidth", screen_width());
            glow_material.set_uniform("screenHeight", screen_height());
            glow_material.set_uniform("Glow", glow); 
            glow_material.set_uniform("maskWidth", cat_tex.width());
            glow_material.set_uniform("maskHeight", cat_tex.height());
            glow_material.set_uniform("maskPos", cat_center);
            glow_material.set_uniform("maskSize", vec2(cat_w, cat_h));
            glow_material.set_uniform("Darkness", darkness);

            let cam = Camera2D {
                target: vec2(screen_width() * 0.5, screen_height() * 0.5),
                zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
                render_target: Some(glow_rt.clone()),
                ..Default::default()
            };
            set_camera(&cam);

            gl_use_material(&glow_material);
            draw_texture_ex(
                &scene_rt.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
            gl_use_default_material();
            set_default_camera();
        }

        {
            // Bind the composite material and give it the three textures
            composite_material.set_texture("ambient_tex", ambient_rt.texture.clone());
            composite_material.set_texture("spot_tex", spot_rt.texture.clone());
            composite_material.set_texture("glow_tex", glow_rt.texture.clone());

            gl_use_material(&composite_material);
            draw_texture_ex(
                &ambient_rt.texture, // any texture works for size; we just need a quad
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
            gl_use_default_material();
            set_default_camera();
        }

        next_frame().await
    }
}

const VERTEX_SHADER: &str = r#"
#version 100
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

const AMB_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;
uniform sampler2D tex;
uniform float Darkness;

void main() {
    vec4 base = texture2D(tex, uv);
    vec3 scene = base.rgb;

    vec3 darkened = mix(scene, vec3(0.0), Darkness);

    gl_FragColor = vec4(clamp(darkened, 0.0, 1.0), base.a);
}
"#;

const SPOT_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;
uniform sampler2D tex;
uniform float Brightness;
uniform vec3 Color;
uniform float ColorIntensity;
uniform float Radius;
uniform float Spread;
uniform vec2 LightPos;
uniform float screenWidth;
uniform float screenHeight;
uniform float Darkness;
uniform float SpotAlpha; 

void main() {
    vec4  base  = texture2D(tex, uv);
    vec3  scene = base.rgb;

    vec2 fragPos = uv * vec2(screenWidth, screenHeight);
    float dist   = distance(fragPos, LightPos);
    float mask   = 1.0 - smoothstep(Radius, Radius + Spread, dist);
    mask = mask * SpotAlpha;

    vec3 tinted = mix(scene, Color, ColorIntensity);
    vec3 lit    = mix(scene, tinted, mask);
    lit += Brightness * Color * mask;

    vec3 contribution = (lit - scene * (1.0 - Darkness)) * mask;
    gl_FragColor = vec4(contribution, mask);
}
"#;

const GLOW_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;

uniform sampler2D scene_tex;
uniform sampler2D tex_mask;

uniform float Brightness;
uniform vec3 Color;
uniform float ColorIntensity;
uniform vec2 LightPos;
uniform float Glow;
uniform float maskWidth;
uniform float maskHeight;
uniform vec2 maskPos;
uniform vec2 maskSize;
uniform float screenWidth;
uniform float screenHeight;
uniform float Darkness; 

float sampleMask(vec2 uvMask)
{
    if (uvMask.x < 0.0 || uvMask.x > 1.0 ||
        uvMask.y < 0.0 || uvMask.y > 1.0) {
        return 0.0;
    }
    return texture2D(tex_mask, uvMask).a;
}

void main() {
    vec4 base = texture2D(scene_tex, uv);
    vec3 scene = base.rgb;
    float finalMask = 1.0;

    vec2 fragScreen = uv * vec2(screenWidth, screenHeight);

    vec2 rel = (fragScreen - (maskPos - maskSize * 0.5)) / maskSize;

    float c00 = sampleMask(rel);

    vec2 pixelSize = vec2(1.0 / maskWidth, 1.0 / maskHeight);

    float sum = 0.0;

    sum += sampleMask(rel + pixelSize * vec2(-Glow, -Glow));
    sum += sampleMask(rel + pixelSize * vec2( 0.0,  -Glow));
    sum += sampleMask(rel + pixelSize * vec2( Glow, -Glow));
    sum += sampleMask(rel + pixelSize * vec2( Glow,  0.0));
    sum += c00; // center
    sum += sampleMask(rel + pixelSize * vec2( Glow,  Glow));
    sum += sampleMask(rel + pixelSize * vec2( 0.0,  Glow));
    sum += sampleMask(rel + pixelSize * vec2(-Glow,  Glow));
    sum += sampleMask(rel + pixelSize * vec2(-Glow,  0.0));
    float avg = sum / 9.0;

    const float SPREAD_TO_BLEND = 1.0;
    float s = clamp(Glow / SPREAD_TO_BLEND, 0.0, 1.0);
    float blurred = mix(c00, avg, s);

    finalMask = max(c00, blurred);

    // Apply lighting
    vec3 tinted = mix(scene, Color, ColorIntensity);
    vec3 lit = mix(scene, tinted, finalMask);
    lit += Brightness * Color * finalMask;

    vec3 contribution = (lit - scene * (1.0 - Darkness)) * finalMask;

    gl_FragColor = vec4(contribution, finalMask);
}
"#;

const COMPOSITE_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;

uniform sampler2D ambient_tex;
uniform sampler2D spot_tex;
uniform sampler2D glow_tex;

void main() {
    vec4 ambient = texture2D(ambient_tex, uv);
    vec4 spot = texture2D(spot_tex, uv);
    vec4 glow = texture2D(glow_tex, uv);

    gl_FragColor = clamp(ambient + spot + glow, 0.0, 1.0);
}
"#;