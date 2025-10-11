use macroquad::prelude::*;

#[macroquad::main("Lighting Example")]
async fn main() {
    let render_target = render_target(screen_width() as u32, screen_height() as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    let circle_tex = load_texture("/Users/charliesovn/Projects/Aseprite/Entity1.png").await.unwrap();

    let cat_tex = load_texture("/Users/charliesovn/Projects/Aseprite/cat.png").await.unwrap();
    cat_tex.set_filter(FilterMode::Nearest);
    
    let material = load_material(
        ShaderSource::Glsl { 
            vertex: VERTEX_SHADER, 
            fragment: FRAGMENT_SHADER 
        },
        MaterialParams { 
            uniforms: vec![
                UniformDesc::new("useMask", UniformType::Int1),
                UniformDesc::new("Darkness", UniformType::Float1),
                UniformDesc::new("Brightness", UniformType::Float1),
                UniformDesc::new("Color", UniformType::Float3),
                UniformDesc::new("ColorIntensity", UniformType::Float1),
                UniformDesc::new("Radius", UniformType::Float1),
                UniformDesc::new("Spread", UniformType::Float1),
                UniformDesc::new("LightPos", UniformType::Float2),
                UniformDesc::new("screenWidth", UniformType::Float1),
                UniformDesc::new("screenHeight", UniformType::Float1),
                // Mask specific params
                UniformDesc::new("Glow", UniformType::Float1),
                UniformDesc::new("maskWidth", UniformType::Float1),
                UniformDesc::new("maskHeight", UniformType::Float1),
                UniformDesc::new("maskPos", UniformType::Float2),
                UniformDesc::new("maskSize", UniformType::Float2),
            ],
            textures: vec!["scene_tex".to_string(), "tex_mask".to_string()],
            ..Default::default() 
        }
    ).unwrap();

    loop {
        // Draw scene to render target
        let pixel_cam = Camera2D {
            target: vec2(screen_width() * 0.5, screen_height() * 0.5),
            zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
            render_target: Some(render_target.clone()),
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

        // Update material textures
        material.set_texture("scene_tex", render_target.texture.clone());
        material.set_texture("tex_mask", cat_tex.clone());

        let (mx, my) = mouse_position();

        // Set lighting parameters
        material.set_uniform("useMask", 1i32); // 0 off/1 on
        material.set_uniform("Darkness", 0.8f32);
        material.set_uniform("ColorIntensity", 0.3f32);
        material.set_uniform("Brightness", 0.2f32);
        material.set_uniform("Color", vec3(1.0, 0.85, 0.6));
        material.set_uniform("Radius", 30.0f32);        
        material.set_uniform("Spread", 100.0f32); 
        material.set_uniform("LightPos", vec2(mx, my));
        material.set_uniform("screenWidth", screen_width());
        material.set_uniform("screenHeight", screen_height());
        
        // Mask specific params
        material.set_uniform("Glow", 1.0f32); 
        material.set_uniform("maskWidth", cat_tex.width());
        material.set_uniform("maskHeight", cat_tex.height());
        material.set_uniform("maskPos", cat_center);
        material.set_uniform("maskSize", vec2(cat_w, cat_h));

        gl_use_material(&material);
        draw_rectangle(0., 0., screen_width(), screen_height(), WHITE);
        gl_use_default_material();

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

const FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;

uniform sampler2D scene_tex;
uniform sampler2D tex_mask;

uniform int useMask;
uniform float Darkness;
uniform float Brightness;
uniform vec3 Color;
uniform float ColorIntensity;
uniform float Radius;
uniform float Spread;
uniform vec2 LightPos;
uniform float screenWidth;
uniform float screenHeight;

// Mask specific uniforms
uniform float Glow;
uniform float maskWidth;
uniform float maskHeight;
uniform vec2 maskPos;
uniform vec2 maskSize;

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

    if (useMask == 1)
    {
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
    } else {
        // Spotlight
        // Convert UV to screen-space position
        vec2 fragPos = uv * vec2(screenWidth, screenHeight);

        // Compute distance from fragment to LightPos
        float dist = distance(fragPos, LightPos);

        // Convert pixel distances to normalized falloff
        // (this ensures that Radius and Spread behave in pixels)
        float lightMask = 1.0 - smoothstep(Radius, Radius + Spread, dist);

        finalMask = lightMask;
    }

    // Apply lighting
    // Darken only the background
    // Background without any lighting
    vec3 background = scene * (1.0 - Darkness);

    vec3 tinted = mix(scene, Color, ColorIntensity);
    vec3 lit = mix(scene, tinted, finalMask);
    lit += Brightness * Color * finalMask;

    vec3 result = mix(background, lit, finalMask);

    gl_FragColor = vec4(clamp(result, 0.0, 1.0), base.a);
}
"#;