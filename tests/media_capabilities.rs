/*use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_test::*;
use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen_test]
async fn test_audio() {
    use fingerprint_rs::media_capabilities::MediaCapabilitiesFingerPrint;
    let mc =     web_sys::window().unwrap().navigator().media_capabilities();
    let result = MediaCapabilitiesFingerPrint::new(mc).await.audio_result;
    let len = result.len();
    let calls = len/3;
    let true_c = result.into_iter().filter(|b|*b).collect::<Vec<bool>>().len();
    panic!("
        len : {len} \n
        calls : {calls} \n
        true_c : {true_c} \n
    ");

}*/
