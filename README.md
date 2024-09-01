<h1>fingerprint-rs</h1>
<p>
fingerprint-rs is rust library that uses wasm-bindgen to create browser fingerprints.
</p>

<p>
It uses web_sys to call browser api to find the browser fingerprint of the users device.
</p>


<p>
How to use it
</p>

```rust
let fingerprint = fingerprint_rs::FingerPrint();
```

<p>
It includes, webgl, canvas, audio, font fingerprinting, window, permissions, and more. If you have any issues let me know via issues.
</p>

<p>
We're currently missing a live deployment to collect enough data for base entropy values, a full fledged example, and an explanation of how to find the bits of information given mutual information and collected data.
</p>

