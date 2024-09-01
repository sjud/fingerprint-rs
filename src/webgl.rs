use std::{
    cmp::{max, min},
    hash::{DefaultHasher, Hasher},
    i32,
};

use js_sys::{Float32Array, Int32Array, Uint32Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, ImageData, WebGl2RenderingContext, WebglDebugRendererInfo};

use super::*;

#[derive(Debug, Clone, Default)]
pub struct WebGLFingerPrint {
    pub renderer: Option<String>,
    pub context_attributes: Option<WebGlContextAttributesFingerPrint>,
    pub shader_precision: Option<ShaderPrecisionFingerPrint>,
    pub supported_extensions: Vec<String>,
    pub parameters: Option<WebGLParametersFingerPrint>,
    pub webgl_image_hash: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct WebGlContextAttributesFingerPrint {
    pub alpha_buffer: Option<bool>,
    pub depth_buffer: Option<bool>,
    pub stencil_buffer: Option<bool>,
    pub anti_aliasing: Option<bool>,
    pub major_performance_caveat: Option<bool>,
    pub power_preference: Option<u32>,
    pub pre_multiplied_alpha: Option<bool>,
    pub preserve_drawing_buffer: Option<bool>,
}
impl WebGlContextAttributesFingerPrint {
    pub fn new(gl: &WebGl2RenderingContext) -> Option<Self> {
        let attr = gl.get_context_attributes()?;

        Some(Self {
            alpha_buffer: attr.get_alpha(),
            depth_buffer: attr.get_depth(),
            stencil_buffer: attr.get_stencil(),
            anti_aliasing: attr.get_antialias(),
            major_performance_caveat: attr.get_fail_if_major_performance_caveat(),
            power_preference: attr.get_power_preference().map(|pp| pp as u32),
            pre_multiplied_alpha: attr.get_premultiplied_alpha(),
            preserve_drawing_buffer: attr.get_preserve_drawing_buffer(),
        })
    }
}
impl WebGLFingerPrint {
    pub fn new(window: &Window) -> Option<Self> {
        let canvas = window
            .document()?
            .create_element("canvas")
            .ok()?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?;
        let ctx = canvas.get_context("webgl2").ok()??;
        let gl = ctx.dyn_into::<WebGl2RenderingContext>().ok()?;
        let renderer = renderer(&gl);
        let supported_extensions = gl
            .get_supported_extensions()?
            .into_iter()
            .map(|s| s.as_string().unwrap_or_default())
            .collect::<Vec<String>>();
        let parameters = WebGLParametersFingerPrint::new(&gl);
        let context_attributes = WebGlContextAttributesFingerPrint::new(&gl);
        let shader_precision = ShaderPrecisionFingerPrint::new(&gl);

        let canvas = window
            .document()?
            .create_element("canvas")
            .ok()?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?;
        let webgl_image_hash = webgl_img_hash(&gl, &canvas);
        Some(Self {
            renderer,
            context_attributes,
            shader_precision,
            supported_extensions,
            parameters,
            webgl_image_hash,
        })
    }
}
fn renderer(gl: &WebGl2RenderingContext) -> Option<String> {
    // firefox has deprecated the WEBGL_debug_render_info. But webkit has not, but it needs to be accessed through the debug. Doesn't work on safari.
    let renderer = if !USER_AGENT.contains("applewebkit") {
        gl.get_parameter(web_sys::WebGl2RenderingContext::RENDERER)
            .ok()?
            .as_string()?
    } else {
        // you need to call get_extension before you can use the extensions constants.
        _ = gl.get_extension("WEBGL_debug_renderer_info");
        gl.get_parameter(WebglDebugRendererInfo::UNMASKED_RENDERER_WEBGL)
            .ok()?
            .as_string()?
    };
    Some(renderer)
}
/// Return's the img hash
pub fn webgl_img_hash(gl: &WebGl2RenderingContext, canvas: &HtmlCanvasElement) -> Option<u64> {
    let vertex_shader_src = r#"
          attribute vec2 position;
          void main() {
              gl_Position = vec4(position, 0.0, 1.0);
          }
      "#;
    let fragment_shader_src = r#"
      precision mediump float;
          void main() {
              gl_FragColor = vec4(0.812, 0.195, 0.553, 0.921); // Set line color
          }
    "#;
    let vertex_shader = gl.create_shader(WebGl2RenderingContext::VERTEX_SHADER)?;
    let fragment_shader = gl.create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)?;

    gl.shader_source(&vertex_shader, vertex_shader_src);
    gl.shader_source(&fragment_shader, fragment_shader_src);

    gl.compile_shader(&vertex_shader);
    gl.compile_shader(&fragment_shader);

    let program = gl.create_program()?;

    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);

    gl.link_program(&program);

    gl.use_program(Some(&program));

    // Set up vertices to form lines
    let num_spokes = 137.;
    let mut vertices: [f32; 548] = [0.; 548];
    let angle_increment = (2. * std::f32::consts::PI) / num_spokes;

    for i in 0..(num_spokes as usize) {
        let f = i as f32;
        let angle = f * angle_increment;
        // define two points for each line
        vertices[i * 4] = 0.; // center x
        vertices[i * 4 + 1] = 0.; // center y
        vertices[i * 4 + 2] = angle.cos() * (canvas.width() as f32 / 2.); //endpoint x
        vertices[i * 4 + 3] = angle.sin() * (canvas.height() as f32 / 2.); //endpoint y
    }

    let vertex_buffer = gl.create_buffer()?;

    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        Float32Array::from(vertices.as_slice()).as_ref(),
        WebGl2RenderingContext::STATIC_DRAW,
    );

    let position_attribute = gl.get_attrib_location(&program, "position") as u32;
    gl.enable_vertex_attrib_array(position_attribute);
    gl.vertex_attrib_pointer_with_i32(
        position_attribute,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );

    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
    gl.clear_color(0., 0., 0., 1.);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    gl.draw_arrays(WebGl2RenderingContext::LINES, 0, num_spokes as i32 * 2);

    let s = canvas.to_data_url().ok()?;
    let mut hasher = DefaultHasher::new();
    hasher.write(s.as_bytes());
    let hash = hasher.finish();

    Some(hash)
}

#[derive(Clone, Debug, Default)]
pub struct ShaderPrecisionFingerPrint {
    least_min: i32,
    most_max: i32,
    highest_precision: i32,
}
impl ShaderPrecisionFingerPrint {
    pub fn new(gl: &WebGl2RenderingContext) -> Option<Self> {
        let shader_types = [
            WebGl2RenderingContext::FRAGMENT_SHADER,
            WebGl2RenderingContext::VERTEX_SHADER,
        ];
        let precision_types = [
            WebGl2RenderingContext::LOW_FLOAT,
            WebGl2RenderingContext::MEDIUM_FLOAT,
            WebGl2RenderingContext::HIGH_FLOAT,
            WebGl2RenderingContext::LOW_INT,
            WebGl2RenderingContext::MEDIUM_INT,
            WebGl2RenderingContext::HIGH_INT,
        ];
        let mut least_min = i32::MAX;
        let mut most_max = i32::MIN;
        let mut highest_precision = i32::MIN;
        for shader in shader_types {
            for precision in precision_types {
                let shader_format = gl.get_shader_precision_format(shader, precision)?;
                let precision = shader_format.precision();
                let range_max = shader_format.range_max();
                let range_min = shader_format.range_min();
                least_min = min(least_min, range_min);
                most_max = max(most_max, range_max);
                highest_precision = max(precision, highest_precision);
            }
        }
        Some(ShaderPrecisionFingerPrint {
            least_min,
            most_max,
            highest_precision,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct WebGLParametersFingerPrint {
    pub active_texture: u32,
    pub aliased_line_width_range: [f32; 2],
    pub aliased_point_size_range: [f32; 2],
    pub alpha_bits: i32,
    pub blend: bool,
    pub blue_bits: i32,
    pub color_clear_value: [f32; 4],
    pub color_writemask: [bool; 4],
    pub compressed_texture_formats: Vec<u32>,
    pub cull_face: bool,
    pub cull_face_mode: u32,
    pub depth_bits: i32,
    pub depth_clear_value: f32,
    pub depth_func: u32,
    pub depth_range: [f32; 2],
    pub depth_test: bool,
    pub depth_writemask: bool,
    pub dither: bool,
    pub front_face: u32,
    pub generate_mipmap_hint: u32,
    pub green_bits: i32,
    pub implementation_color_read_format: u32,
    pub implementation_color_read_type: u32,
    pub line_width: f32,
    pub max_combined_texture_image_units: i32,
    pub max_cube_map_texture_size: i32,
    pub max_fragment_uniform_vectors: i32,
    pub max_renderbuffer_size: i32,
    pub max_texture_image_units: i32,
    pub max_texture_size: i32,
    pub max_varying_vectors: i32,
    pub max_vertex_attribs: i32,
    pub max_vertex_texture_image_units: i32,
    pub max_vertex_uniform_vectors: i32,
    pub max_viewport_dims: [i32; 2],
    pub pack_alignment: i32,
    pub polygon_offset_factor: f32,
    pub polygon_offset_fill: bool,
    pub polygon_offset_units: f32,
    pub red_bits: i32,
    pub renderer: String,
    pub sample_buffers: i32,
    pub sample_coverage_invert: bool,
    pub sample_coverage_value: f32,
    pub samples: i32,
    pub scissor_box: [i32; 4],
    pub scissor_test: bool,
    pub shading_language_version: String,
    pub stencil_back_fail: u32,
    pub stencil_back_func: u32,
    pub stencil_back_pass_depth_fail: u32,
    pub stencil_back_pass_depth_pass: u32,
    pub stencil_back_ref: i32,
    pub stencil_back_value_mask: u32,
    pub stencil_back_writemask: u32,
    pub stencil_bits: i32,
    pub stencil_clear_value: i32,
    pub stencil_fail: u32,
    pub stencil_func: u32,
    pub stencil_pass_depth_fail: u32,
    pub stencil_pass_depth_pass: u32,
    pub stencil_ref: i32,
    pub stencil_test: bool,
    pub stencil_value_mask: u32,
    pub stencil_writemask: u32,
    pub subpixel_bits: i32,
    pub unpack_alignment: i32,
    pub unpack_colorspace_conversion_webgl: u32,
    pub unpack_flip_y_webgl: bool,
    pub unpack_premultiply_alpha_webgl: bool,
    pub vendor: String,
    pub version: String,
    pub viewport: [i32; 4],
}

impl WebGLParametersFingerPrint {
    pub fn new(gl: &WebGl2RenderingContext) -> Option<Self> {
        Some(Self {
            active_texture: gl
                .get_parameter(WebGl2RenderingContext::ACTIVE_TEXTURE)
                .ok()?
                .as_f64()? as u32,

            aliased_line_width_range: {
                let array = Float32Array::new(
                    &gl.get_parameter(WebGl2RenderingContext::ALIASED_LINE_WIDTH_RANGE)
                        .ok()?,
                );
                [array.get_index(0), array.get_index(1)]
            },

            aliased_point_size_range: {
                let array = Float32Array::new(
                    &gl.get_parameter(WebGl2RenderingContext::ALIASED_POINT_SIZE_RANGE)
                        .ok()?,
                );
                [array.get_index(0), array.get_index(1)]
            },

            alpha_bits: gl
                .get_parameter(WebGl2RenderingContext::ALPHA_BITS)
                .ok()?
                .as_f64()? as i32,

            blend: gl
                .get_parameter(WebGl2RenderingContext::BLEND)
                .ok()?
                .as_bool()?,

            blue_bits: gl
                .get_parameter(WebGl2RenderingContext::BLUE_BITS)
                .ok()?
                .as_f64()? as i32,

            color_clear_value: {
                let array = Float32Array::new(
                    &gl.get_parameter(WebGl2RenderingContext::COLOR_CLEAR_VALUE)
                        .ok()?,
                );
                [
                    array.get_index(0),
                    array.get_index(1),
                    array.get_index(2),
                    array.get_index(3),
                ]
            },

            color_writemask: {
                let array = js_sys::Array::from(
                    &gl.get_parameter(WebGl2RenderingContext::COLOR_WRITEMASK)
                        .ok()?,
                );
                [
                    array.get(0).as_bool()?,
                    array.get(1).as_bool()?,
                    array.get(2).as_bool()?,
                    array.get(3).as_bool()?,
                ]
            },

            compressed_texture_formats: {
                let array = Uint32Array::new(
                    &gl.get_parameter(WebGl2RenderingContext::COMPRESSED_TEXTURE_FORMATS)
                        .ok()?,
                );
                array.to_vec()
            },

            cull_face: gl
                .get_parameter(WebGl2RenderingContext::CULL_FACE)
                .ok()?
                .as_bool()?,

            cull_face_mode: gl
                .get_parameter(WebGl2RenderingContext::CULL_FACE_MODE)
                .ok()?
                .as_f64()? as u32,

            depth_bits: gl
                .get_parameter(WebGl2RenderingContext::DEPTH_BITS)
                .ok()?
                .as_f64()? as i32,

            depth_clear_value: gl
                .get_parameter(WebGl2RenderingContext::DEPTH_CLEAR_VALUE)
                .ok()?
                .as_f64()? as f32,

            depth_func: gl
                .get_parameter(WebGl2RenderingContext::DEPTH_FUNC)
                .ok()?
                .as_f64()? as u32,

            depth_range: {
                let array =
                    Float32Array::new(&gl.get_parameter(WebGl2RenderingContext::DEPTH_RANGE).ok()?);
                [array.get_index(0), array.get_index(1)]
            },

            depth_test: gl
                .get_parameter(WebGl2RenderingContext::DEPTH_TEST)
                .ok()?
                .as_bool()?,

            depth_writemask: gl
                .get_parameter(WebGl2RenderingContext::DEPTH_WRITEMASK)
                .ok()?
                .as_bool()?,

            dither: gl
                .get_parameter(WebGl2RenderingContext::DITHER)
                .ok()?
                .as_bool()?,

            front_face: gl
                .get_parameter(WebGl2RenderingContext::FRONT_FACE)
                .ok()?
                .as_f64()? as u32,

            generate_mipmap_hint: gl
                .get_parameter(WebGl2RenderingContext::GENERATE_MIPMAP_HINT)
                .ok()?
                .as_f64()? as u32,

            green_bits: gl
                .get_parameter(WebGl2RenderingContext::GREEN_BITS)
                .ok()?
                .as_f64()? as i32,

            implementation_color_read_format: gl
                .get_parameter(WebGl2RenderingContext::IMPLEMENTATION_COLOR_READ_FORMAT)
                .ok()?
                .as_f64()? as u32,

            implementation_color_read_type: gl
                .get_parameter(WebGl2RenderingContext::IMPLEMENTATION_COLOR_READ_TYPE)
                .ok()?
                .as_f64()? as u32,

            line_width: gl
                .get_parameter(WebGl2RenderingContext::LINE_WIDTH)
                .ok()?
                .as_f64()? as f32,

            max_combined_texture_image_units: gl
                .get_parameter(WebGl2RenderingContext::MAX_COMBINED_TEXTURE_IMAGE_UNITS)
                .ok()?
                .as_f64()? as i32,

            max_cube_map_texture_size: gl
                .get_parameter(WebGl2RenderingContext::MAX_CUBE_MAP_TEXTURE_SIZE)
                .ok()?
                .as_f64()? as i32,

            max_fragment_uniform_vectors: gl
                .get_parameter(WebGl2RenderingContext::MAX_FRAGMENT_UNIFORM_VECTORS)
                .ok()?
                .as_f64()? as i32,

            max_renderbuffer_size: gl
                .get_parameter(WebGl2RenderingContext::MAX_RENDERBUFFER_SIZE)
                .ok()?
                .as_f64()? as i32,

            max_texture_image_units: gl
                .get_parameter(WebGl2RenderingContext::MAX_TEXTURE_IMAGE_UNITS)
                .ok()?
                .as_f64()? as i32,

            max_texture_size: gl
                .get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)
                .ok()?
                .as_f64()? as i32,

            max_varying_vectors: gl
                .get_parameter(WebGl2RenderingContext::MAX_VARYING_VECTORS)
                .ok()?
                .as_f64()? as i32,

            max_vertex_attribs: gl
                .get_parameter(WebGl2RenderingContext::MAX_VERTEX_ATTRIBS)
                .ok()?
                .as_f64()? as i32,

            max_vertex_texture_image_units: gl
                .get_parameter(WebGl2RenderingContext::MAX_VERTEX_TEXTURE_IMAGE_UNITS)
                .ok()?
                .as_f64()? as i32,

            max_vertex_uniform_vectors: gl
                .get_parameter(WebGl2RenderingContext::MAX_VERTEX_UNIFORM_VECTORS)
                .ok()?
                .as_f64()? as i32,

            max_viewport_dims: {
                let array = Int32Array::new(
                    &gl.get_parameter(WebGl2RenderingContext::MAX_VIEWPORT_DIMS)
                        .ok()?,
                );
                [array.get_index(0), array.get_index(1)]
            },

            pack_alignment: gl
                .get_parameter(WebGl2RenderingContext::PACK_ALIGNMENT)
                .ok()?
                .as_f64()? as i32,

            polygon_offset_factor: gl
                .get_parameter(WebGl2RenderingContext::POLYGON_OFFSET_FACTOR)
                .ok()?
                .as_f64()? as f32,

            polygon_offset_fill: gl
                .get_parameter(WebGl2RenderingContext::POLYGON_OFFSET_FILL)
                .ok()?
                .as_bool()?,

            polygon_offset_units: gl
                .get_parameter(WebGl2RenderingContext::POLYGON_OFFSET_UNITS)
                .ok()?
                .as_f64()? as f32,

            red_bits: gl
                .get_parameter(WebGl2RenderingContext::RED_BITS)
                .ok()?
                .as_f64()? as i32,

            renderer: gl
                .get_parameter(WebGl2RenderingContext::RENDERER)
                .ok()?
                .as_string()?,

            sample_buffers: gl
                .get_parameter(WebGl2RenderingContext::SAMPLE_BUFFERS)
                .ok()?
                .as_f64()? as i32,

            sample_coverage_invert: gl
                .get_parameter(WebGl2RenderingContext::SAMPLE_COVERAGE_INVERT)
                .ok()?
                .as_bool()?,

            sample_coverage_value: gl
                .get_parameter(WebGl2RenderingContext::SAMPLE_COVERAGE_VALUE)
                .ok()?
                .as_f64()? as f32,

            samples: gl
                .get_parameter(WebGl2RenderingContext::SAMPLES)
                .ok()?
                .as_f64()? as i32,

            scissor_box: {
                let array =
                    Int32Array::new(&gl.get_parameter(WebGl2RenderingContext::SCISSOR_BOX).ok()?);
                [
                    array.get_index(0),
                    array.get_index(1),
                    array.get_index(2),
                    array.get_index(3),
                ]
            },

            scissor_test: gl
                .get_parameter(WebGl2RenderingContext::SCISSOR_TEST)
                .ok()?
                .as_bool()?,

            shading_language_version: gl
                .get_parameter(WebGl2RenderingContext::SHADING_LANGUAGE_VERSION)
                .ok()?
                .as_string()?,

            stencil_back_fail: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_BACK_FAIL)
                .ok()?
                .as_f64()? as u32,

            stencil_back_func: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_BACK_FUNC)
                .ok()?
                .as_f64()? as u32,

            stencil_back_pass_depth_fail: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_BACK_PASS_DEPTH_FAIL)
                .ok()?
                .as_f64()? as u32,

            stencil_back_pass_depth_pass: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_BACK_PASS_DEPTH_PASS)
                .ok()?
                .as_f64()? as u32,

            stencil_back_ref: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_BACK_REF)
                .ok()?
                .as_f64()? as i32,

            stencil_back_value_mask: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_BACK_VALUE_MASK)
                .ok()?
                .as_f64()? as u32,

            stencil_back_writemask: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_BACK_WRITEMASK)
                .ok()?
                .as_f64()? as u32,

            stencil_bits: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_BITS)
                .ok()?
                .as_f64()? as i32,

            stencil_clear_value: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_CLEAR_VALUE)
                .ok()?
                .as_f64()? as i32,

            stencil_fail: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_FAIL)
                .ok()?
                .as_f64()? as u32,

            stencil_func: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_FUNC)
                .ok()?
                .as_f64()? as u32,

            stencil_pass_depth_fail: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_PASS_DEPTH_FAIL)
                .ok()?
                .as_f64()? as u32,

            stencil_pass_depth_pass: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_PASS_DEPTH_PASS)
                .ok()?
                .as_f64()? as u32,

            stencil_ref: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_REF)
                .ok()?
                .as_f64()? as i32,

            stencil_test: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_TEST)
                .ok()?
                .as_bool()?,

            stencil_value_mask: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_VALUE_MASK)
                .ok()?
                .as_f64()? as u32,

            stencil_writemask: gl
                .get_parameter(WebGl2RenderingContext::STENCIL_WRITEMASK)
                .ok()?
                .as_f64()? as u32,

            subpixel_bits: gl
                .get_parameter(WebGl2RenderingContext::SUBPIXEL_BITS)
                .ok()?
                .as_f64()? as i32,

            unpack_alignment: gl
                .get_parameter(WebGl2RenderingContext::UNPACK_ALIGNMENT)
                .ok()?
                .as_f64()? as i32,

            unpack_colorspace_conversion_webgl: gl
                .get_parameter(WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL)
                .ok()?
                .as_f64()? as u32,

            unpack_flip_y_webgl: gl
                .get_parameter(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL)
                .ok()?
                .as_bool()?,

            unpack_premultiply_alpha_webgl: gl
                .get_parameter(WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL)
                .ok()?
                .as_bool()?,

            vendor: gl
                .get_parameter(WebGl2RenderingContext::VENDOR)
                .ok()?
                .as_string()?,

            version: gl
                .get_parameter(WebGl2RenderingContext::VERSION)
                .ok()?
                .as_string()?,

            viewport: {
                let array =
                    Int32Array::new(&gl.get_parameter(WebGl2RenderingContext::VIEWPORT).ok()?);
                [
                    array.get_index(0),
                    array.get_index(1),
                    array.get_index(2),
                    array.get_index(3),
                ]
            },
        })
    }
}
