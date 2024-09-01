use fingerprint_rs::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test_configure;
use wasm_bindgen_test::*;
use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_audio() {
    let audio_fingerprint = AudioFingerPrint::new(&window().unwrap()).await;
    assert!(audio_fingerprint.is_some());
    console_log!("{:#?}", audio_fingerprint);
}
#[wasm_bindgen_test]
async fn test_screen() {
    let screen = ScreenFingerPrint::new(&window().unwrap().screen().unwrap());
    assert!(screen.is_some());
    console_log!("{screen:#?}");
}
#[wasm_bindgen_test]
async fn test_permissions() {
    let permissions = PermissionFingerPrint::new(&window().unwrap().navigator()).await;
    assert!(permissions.is_some())
}
#[wasm_bindgen_test]
async fn test_network_information() {
    let net_info =
        NetworkInformationFingerPrint::from_result(window().unwrap().navigator().connection());
    assert!(net_info.is_some());
    console_log!("{net_info:#?}");
}
#[wasm_bindgen_test]
async fn test_audio_hash() {
    let hash = audio_hash().await;
    assert!(hash.is_some());
    console_log!("audio_hash: {hash:#?}");
}

#[wasm_bindgen_test]
async fn test_audio_formats() {
    let formats = CheckAudioFormats::new(&window().unwrap());
    assert!(formats.is_some());
    console_log!("{formats:#?}");
}

#[wasm_bindgen_test]
async fn test_navigator() {
    let nav_finger_print = NavigatorFingerPrint::new(window().unwrap().navigator()).await;
    console_log!("{nav_finger_print:#?}");
}

#[wasm_bindgen_test]
async fn test_canvas() {
    let canvas_fingerprint = CanvasFingerPrint::new(&window().unwrap()).unwrap();
    let canvas_fingerprint_2 = CanvasFingerPrint::new(&window().unwrap()).unwrap();
    // check to see if these differ for the same machine/browser after two calls.
    assert_eq!(canvas_fingerprint, canvas_fingerprint_2);
    console_log!("{canvas_fingerprint:#?}");
}

#[wasm_bindgen_test]
async fn test_webgl() {
    let webgl_fingerprint = WebGLFingerPrint::new(&window().unwrap()).unwrap();
    console_log!("{webgl_fingerprint:#?}");
}

#[wasm_bindgen_test]
async fn test_fonts() {
    let fonts = detect_fonts(&window().unwrap().document().unwrap()).unwrap();
    assert!(!fonts.is_empty());
    console_log!("{fonts:#?}");
}

#[wasm_bindgen_test]
async fn test_webgl_attributes() {
    let webgl_attr_fp = WebGlContextAttributesFingerPrint::new(
        &window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .unchecked_into::<HtmlCanvasElement>()
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .unchecked_into::<WebGl2RenderingContext>(),
    );
    assert!(webgl_attr_fp.is_some());
    console_log!("{webgl_attr_fp:#?}");
}

#[wasm_bindgen_test]
async fn test_shader_precision() {
    let shader = ShaderPrecisionFingerPrint::new(
        &window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .unchecked_into::<HtmlCanvasElement>()
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .unchecked_into::<WebGl2RenderingContext>(),
    );
    assert!(shader.is_some());
    console_log!("{shader:#?}");
}

#[wasm_bindgen_test]
async fn test_webgl_parameters() {
    let fp = WebGLParametersFingerPrint::new(
        &window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .unchecked_into::<HtmlCanvasElement>()
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .unchecked_into::<WebGl2RenderingContext>(),
    );
    assert!(fp.is_some());
    console_log!("{fp:#?}");
}
