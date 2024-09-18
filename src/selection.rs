use std::sync::LazyLock;

use chaos_framework::{Renderer, Shader};

pub static SELECTION_VS: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aColor;
layout (location = 2) in vec2 aTexCoord;
layout (location = 3) in vec3 aNormal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;

out vec4 fColor;
out vec3 Normal;
out vec3 FragPos;
out vec2 TexCoord; // Pass texture coordinates to the fragment shader

void main() {
    gl_Position = proj * view * model * vec4(aPos, 1.0);
    TexCoord = aTexCoord; // Pass texture coordinates
    FragPos = vec3(model * vec4(aPos, 1.0));
    Normal = mat3(transpose(inverse(model))) * aNormal;  
}
"#;

pub static SELECTION_FS: &str = r#"
#version 330 core
out vec4 FragColor;

in vec2 TexCoord;

uniform vec3 lightColor[256];
uniform vec3 lightPos[256];  
uniform vec3 viewPos;        
uniform int has_texture;     
uniform int num_lights;      

const int max_iterations = 64;
const float escape_radius = 2.0;

int mandelbrot(vec2 c) {
    vec2 z = vec2(0.0);
    int i;
    for (i = 0; i < max_iterations; i++) {
        float x = (z.x * z.x - z.y * z.y) + c.x;
        float y = (2.0 * z.x * z.y) + c.y;
        z.x = x;
        z.y = y;
        if (dot(z, z) > escape_radius * escape_radius)
            break;
    }
    return i;
}

vec3 calculateNormal(vec2 c, vec2 fragCoord) {
    float epsilon = 1e-3;

    float left  = float(mandelbrot(c - vec2(epsilon, 0.0))) / max_iterations;
    float right = float(mandelbrot(c + vec2(epsilon, 0.0))) / max_iterations;
    float down  = float(mandelbrot(c - vec2(0.0, epsilon))) / max_iterations;
    float up    = float(mandelbrot(c + vec2(0.0, epsilon))) / max_iterations;

    return normalize(vec3(left - right, down - up, 2.0));
}

vec3 calculateLighting(vec3 normal, vec3 fragPos) {
    vec3 result = vec3(0.0);
    vec3 viewDir = normalize(viewPos - fragPos);

    for (int i = 0; i < num_lights; i++) {
        vec3 lightDir = normalize(lightPos[i] - fragPos);

        float diff = max(dot(normal, lightDir), 0.0);

        vec3 reflectDir = reflect(-lightDir, normal);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);

        vec3 diffuse = diff * lightColor[i];
        vec3 specular = spec * lightColor[i];
        result += diffuse + specular;
    }

    return result;
}

void main() {
    vec2 fragCoord = gl_FragCoord.xy;

    vec2 c = (fragCoord - vec2(800.0, 300.0)) / 300.0 * 1.0; 

    int iterations = mandelbrot(c);

    float colorFactor = float(iterations) / float(max_iterations);
    vec3 baseColor = vec3(colorFactor * 0.8, colorFactor * 0.4, colorFactor * 1.0);

    vec3 normal = calculateNormal(c, fragCoord);

    vec3 fragPos = vec3(c, 0.0);

    vec3 lighting = calculateLighting(normal, fragPos);

    vec3 finalColor = baseColor * lighting;

    FragColor = vec4(finalColor, 1.0);
}

"#;

pub static SELECTION_SHADER: LazyLock<Shader> = LazyLock::new(|| {
    Shader::new_pipeline(SELECTION_VS, SELECTION_FS)
});

pub fn update_selection_shader_from_renderer(renderer: &mut Renderer) {
    unsafe { 
        renderer.camera.send_uniforms(&SELECTION_SHADER); 
        renderer.send_light_uniforms(&SELECTION_SHADER);
    };
}