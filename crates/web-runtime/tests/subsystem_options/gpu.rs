use std::sync::Arc;

use web_runtime::{Browser, PageOptions};

use super::support::UnusedLoader;

#[test]
fn webgpu_option_executes_compute_and_texture_transfers_when_adapter_is_available() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuResult = "pending";
        navigator.gpu.requestAdapter({ powerPreference: "high-performance" }).then(adapter => {
            if (!adapter) { gpuResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const canvas = document.createElement("canvas");
                canvas.width = 64;
                canvas.height = 64;
                const canvasContext = canvas.getContext("webgpu");
                canvasContext.configure({ device, format: "rgba8unorm", alphaMode: "premultiplied" });
                const buffer = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
                    mappedAtCreation: true,
                });
                new Uint32Array(buffer.getMappedRange()).set([1, 2, 3, 4]);
                buffer.unmap();
                device.queue.writeBuffer(buffer, 0, new Uint32Array([5, 6, 7, 8]));
                const shader = device.createShaderModule({ code: `
                    @group(0) @binding(0) var<storage, read_write> values: array<u32>;
                    @compute @workgroup_size(1)
                    fn main(@builtin(global_invocation_id) id: vec3<u32>) {
                        values[id.x] = values[id.x] * 2u;
                    }
                ` });
                const pipeline = device.createComputePipeline({
                    layout: "auto",
                    compute: { module: shader, entryPoint: "main" },
                });
                const group = device.createBindGroup({
                    layout: pipeline.getBindGroupLayout(0),
                    entries: [{ binding: 0, resource: { buffer } }],
                });
                const readback = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const textureSource = device.createBuffer({
                    size: 256,
                    usage: GPUBufferUsage.COPY_SRC,
                    mappedAtCreation: true,
                });
                new Uint8Array(textureSource.getMappedRange()).set([11, 22, 33, 44]);
                textureSource.unmap();
                const texture = device.createTexture({
                    size: [64, 1],
                    format: "rgba8unorm",
                    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC,
                });
                const textureReadback = device.createBuffer({
                    size: 256,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const renderShader = device.createShaderModule({ code: `
                    struct Color { value: vec4f }
                    @group(0) @binding(0) var<uniform> color: Color;
                    @vertex fn vertexMain(@location(0) position: vec2f) -> @builtin(position) vec4f {
                        return vec4f(position, 0, 1);
                    }
                    @fragment fn fragmentMain() -> @location(0) vec4f {
                        return color.value;
                    }
                ` });
                const renderPipeline = device.createRenderPipeline({
                    layout: "auto",
                    vertex: {
                        module: renderShader,
                        entryPoint: "vertexMain",
                        buffers: [{
                            arrayStride: 8,
                            attributes: [{ format: "float32x2", offset: 0, shaderLocation: 0 }],
                        }],
                    },
                    fragment: { module: renderShader, entryPoint: "fragmentMain", targets: [{ format: "rgba8unorm" }] },
                    primitive: { topology: "triangle-list" },
                });
                const blendPipeline = device.createRenderPipeline({
                    layout: "auto",
                    vertex: {
                        module: renderShader,
                        entryPoint: "vertexMain",
                        buffers: [{
                            arrayStride: 8,
                            attributes: [{ format: "float32x2", offset: 0, shaderLocation: 0 }],
                        }],
                    },
                    fragment: { module: renderShader, entryPoint: "fragmentMain", targets: [{
                        format: "rgba8unorm",
                        blend: {
                            color: { srcFactor: "src-alpha", dstFactor: "one-minus-src-alpha", operation: "add" },
                            alpha: { srcFactor: "one", dstFactor: "one-minus-src-alpha", operation: "add" },
                        },
                    }] },
                    primitive: { topology: "triangle-list" },
                });
                const renderTexture = device.createTexture({
                    size: [64, 64],
                    format: "rgba8unorm",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
                });
                const renderReadback = device.createBuffer({
                    size: 256 * 64,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const blendTexture = device.createTexture({
                    size: [64, 64],
                    format: "rgba8unorm",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
                });
                const blendReadback = device.createBuffer({
                    size: 256 * 64,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const vertexBuffer = device.createBuffer({
                    size: 32,
                    usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(vertexBuffer, 0, new Float32Array([
                    -1, -1, 1, -1, -1, 1, 1, 1,
                ]));
                const indexBuffer = device.createBuffer({
                    size: 12,
                    usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(indexBuffer, 0, new Uint16Array([0, 1, 2, 2, 1, 3]));
                const colorBuffer = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(colorBuffer, 0, new Float32Array([1, 0, 0, 1]));
                const renderGroup = device.createBindGroup({
                    layout: renderPipeline.getBindGroupLayout(0),
                    entries: [{ binding: 0, resource: { buffer: colorBuffer } }],
                });
                const blendColorBuffer = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(blendColorBuffer, 0, new Float32Array([1, 0, 0, 0.5]));
                const blendGroup = device.createBindGroup({
                    layout: blendPipeline.getBindGroupLayout(0),
                    entries: [{ binding: 0, resource: { buffer: blendColorBuffer } }],
                });
                const sampledTexture = device.createTexture({
                    size: [2, 2],
                    format: "rgba8unorm",
                    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
                });
                device.queue.writeTexture(
                    { texture: sampledTexture },
                    new Uint8Array([
                        255, 0, 0, 255, 0, 255, 0, 255,
                        0, 0, 255, 255, 255, 255, 255, 255,
                    ]),
                    { bytesPerRow: 8, rowsPerImage: 2 },
                    [2, 2],
                );
                const sampler = device.createSampler({ magFilter: "nearest", minFilter: "nearest" });
                const sampleOutput = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
                });
                const sampleReadback = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const sampleShader = device.createShaderModule({ code: `
                    struct Sample { value: vec4f }
                    @group(0) @binding(0) var imageSampler: sampler;
                    @group(0) @binding(1) var image: texture_2d<f32>;
                    @group(0) @binding(2) var<storage, read_write> output: Sample;
                    @compute @workgroup_size(1)
                    fn main() {
                        output.value = textureSampleLevel(image, imageSampler, vec2f(0.25, 0.25), 0.0);
                    }
                ` });
                const samplePipeline = device.createComputePipeline({
                    layout: "auto",
                    compute: { module: sampleShader, entryPoint: "main" },
                });
                const sampleGroup = device.createBindGroup({
                    layout: samplePipeline.getBindGroupLayout(0),
                    entries: [
                        { binding: 0, resource: sampler },
                        { binding: 1, resource: sampledTexture.createView() },
                        { binding: 2, resource: { buffer: sampleOutput } },
                    ],
                });
                const storageTexture = device.createTexture({
                    size: [1, 1],
                    format: "rgba8unorm",
                    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.COPY_SRC,
                });
                const storageReadback = device.createBuffer({
                    size: 256,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const storageShader = device.createShaderModule({ code: `
                    @group(0) @binding(0) var output: texture_storage_2d<rgba8unorm, write>;
                    @compute @workgroup_size(1)
                    fn main() {
                        textureStore(output, vec2i(0, 0), vec4f(0, 1, 0, 1));
                    }
                ` });
                const storagePipeline = device.createComputePipeline({
                    layout: "auto",
                    compute: { module: storageShader, entryPoint: "main" },
                });
                const storageGroup = device.createBindGroup({
                    layout: storagePipeline.getBindGroupLayout(0),
                    entries: [{ binding: 0, resource: storageTexture.createView() }],
                });
                const encoder = device.createCommandEncoder();
                const pass = encoder.beginComputePass();
                pass.setPipeline(pipeline);
                pass.setBindGroup(0, group);
                pass.dispatchWorkgroups(4);
                pass.end();
                const samplePass = encoder.beginComputePass();
                samplePass.setPipeline(samplePipeline);
                samplePass.setBindGroup(0, sampleGroup);
                samplePass.dispatchWorkgroups(1);
                samplePass.end();
                const storagePass = encoder.beginComputePass();
                storagePass.setPipeline(storagePipeline);
                storagePass.setBindGroup(0, storageGroup);
                storagePass.dispatchWorkgroups(1);
                storagePass.end();
                const renderPass = encoder.beginRenderPass({ colorAttachments: [{
                    view: renderTexture.createView(),
                    loadOp: "clear",
                    clearValue: [0, 0, 0, 1],
                    storeOp: "store",
                }] });
                renderPass.setPipeline(renderPipeline);
                renderPass.setBindGroup(0, renderGroup);
                renderPass.setVertexBuffer(0, vertexBuffer);
                renderPass.setIndexBuffer(indexBuffer, "uint16");
                renderPass.drawIndexed(6);
                renderPass.end();
                const blendPass = encoder.beginRenderPass({ colorAttachments: [{
                    view: blendTexture.createView(),
                    loadOp: "clear",
                    clearValue: [0, 0, 1, 1],
                    storeOp: "store",
                }] });
                blendPass.setPipeline(blendPipeline);
                blendPass.setBindGroup(0, blendGroup);
                blendPass.setVertexBuffer(0, vertexBuffer);
                blendPass.setIndexBuffer(indexBuffer, "uint16");
                blendPass.drawIndexed(6);
                blendPass.end();
                const canvasPass = encoder.beginRenderPass({ colorAttachments: [{
                    view: canvasContext.getCurrentTexture().createView(),
                    loadOp: "clear",
                    clearValue: [0, 0, 0, 1],
                    storeOp: "store",
                }] });
                canvasPass.setPipeline(renderPipeline);
                canvasPass.setBindGroup(0, renderGroup);
                canvasPass.setVertexBuffer(0, vertexBuffer);
                canvasPass.setIndexBuffer(indexBuffer, "uint16");
                canvasPass.drawIndexed(6);
                canvasPass.end();
                encoder.copyBufferToBuffer(buffer, 0, readback, 0, 16);
                encoder.copyBufferToBuffer(sampleOutput, 0, sampleReadback, 0, 16);
                encoder.copyTextureToBuffer(
                    { texture: storageTexture },
                    { buffer: storageReadback, bytesPerRow: 256 },
                    [1, 1],
                );
                encoder.copyBufferToTexture(
                    { buffer: textureSource, bytesPerRow: 256 },
                    { texture },
                    [64, 1],
                );
                encoder.copyTextureToBuffer(
                    { texture: renderTexture },
                    { buffer: renderReadback, bytesPerRow: 256 },
                    [64, 64],
                );
                encoder.copyTextureToBuffer(
                    { texture: blendTexture },
                    { buffer: blendReadback, bytesPerRow: 256 },
                    [64, 64],
                );
                encoder.copyTextureToBuffer(
                    { texture },
                    { buffer: textureReadback, bytesPerRow: 256 },
                    [64, 1],
                );
                const command = encoder.finish();
                device.queue.submit([command]);
                return readback.mapAsync(GPUMapMode.READ).then(() => {
                    const values = [...new Uint32Array(readback.getMappedRange())];
                    readback.unmap();
                    return textureReadback.mapAsync(GPUMapMode.READ).then(() => {
                    const textureValues = [...new Uint8Array(textureReadback.getMappedRange()).slice(0, 4)];
                    textureReadback.unmap();
                    return renderReadback.mapAsync(GPUMapMode.READ).then(() => {
                    const renderValues = [...new Uint8Array(renderReadback.getMappedRange()).slice(0, 4)];
                    renderReadback.unmap();
                    return sampleReadback.mapAsync(GPUMapMode.READ).then(() => {
                    const sampledValues = [...new Float32Array(sampleReadback.getMappedRange())].map(value => Math.round(value * 255));
                    sampleReadback.unmap();
                    return storageReadback.mapAsync(GPUMapMode.READ).then(() => {
                    const storageValues = [...new Uint8Array(storageReadback.getMappedRange()).slice(0, 4)];
                    storageReadback.unmap();
                    return blendReadback.mapAsync(GPUMapMode.READ).then(() => {
                    const blendValues = [...new Uint8Array(blendReadback.getMappedRange()).slice(0, 4)];
                    blendReadback.unmap();
                    gpuResult = JSON.stringify({
                        adapter: adapter instanceof GPUAdapter,
                        device: device instanceof GPUDevice,
                        buffer: buffer instanceof GPUBuffer,
                        encoder: encoder instanceof GPUCommandEncoder,
                        command: command instanceof GPUCommandBuffer,
                        shader: shader instanceof GPUShaderModule,
                        pipeline: pipeline instanceof GPUComputePipeline,
                        group: group instanceof GPUBindGroup,
                        pass: pass instanceof GPUComputePassEncoder,
                        texture: texture instanceof GPUTexture,
                        renderPipeline: renderPipeline instanceof GPURenderPipeline,
                        blendPipeline: blendPipeline instanceof GPURenderPipeline,
                        renderPass: renderPass instanceof GPURenderPassEncoder,
                        sampler: sampler instanceof GPUSampler,
                        sampledTexture: sampledTexture instanceof GPUTexture,
                        sampleGroup: sampleGroup instanceof GPUBindGroup,
                        storageTexture: storageTexture instanceof GPUTexture,
                        storageGroup: storageGroup instanceof GPUBindGroup,
                        canvasContext: canvasContext instanceof GPUCanvasContext,
                        canvasExclusive: canvas.getContext("2d") === null,
                        canvasData: canvas.toDataURL().startsWith("data:image/png;base64,") && canvas.toDataURL().length > 100,
                        values,
                        textureValues,
                        renderValues,
                        sampledValues,
                        storageValues,
                        blendValues,
                        mapState: readback.mapState,
                        maxBufferSize: device.limits.maxBufferSize > 0,
                        native: GPU.prototype.requestAdapter.toString(),
                    });
                    });
                    });
                    });
                    });
                    });
                });
            });
        }).catch(error => gpuResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page.eval("gpuResult").unwrap().to_string().unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"adapter":true,"device":true,"buffer":true,"encoder":true,"command":true,"shader":true,"pipeline":true,"group":true,"pass":true,"texture":true,"renderPipeline":true,"blendPipeline":true,"renderPass":true,"sampler":true,"sampledTexture":true,"sampleGroup":true,"storageTexture":true,"storageGroup":true,"canvasContext":true,"canvasExclusive":true,"canvasData":true,"values":[10,12,14,16],"textureValues":[11,22,33,44],"renderValues":[255,0,0,255],"sampledValues":[255,0,0,255],"storageValues":[0,255,0,255],"blendValues":[128,0,127,255],"mapState":"unmapped","maxBufferSize":true,"native":"function requestAdapter() { [native code] }"}"#
            || result
                == r#"{"adapter":true,"device":true,"buffer":true,"encoder":true,"command":true,"shader":true,"pipeline":true,"group":true,"pass":true,"texture":true,"renderPipeline":true,"blendPipeline":true,"renderPass":true,"sampler":true,"sampledTexture":true,"sampleGroup":true,"storageTexture":true,"storageGroup":true,"canvasContext":true,"canvasExclusive":true,"canvasData":true,"values":[10,12,14,16],"textureValues":[11,22,33,44],"renderValues":[255,0,0,255],"sampledValues":[255,0,0,255],"storageValues":[0,255,0,255],"blendValues":[128,0,128,255],"mapState":"unmapped","maxBufferSize":true,"native":"function requestAdapter() { [native code] }"}"#,
        "unexpected WebGPU result: {result}",
    );
}

#[test]
fn webgpu_maps_buffers_for_write_and_honors_partial_mapping_ranges() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuMapWriteResult = "pending";
        navigator.gpu.requestAdapter().then(async adapter => {
            if (!adapter) { gpuMapWriteResult = "no-adapter"; return; }
            const device = await adapter.requestDevice();
            const source = device.createBuffer({
                size: 16,
                usage: GPUBufferUsage.MAP_WRITE | GPUBufferUsage.COPY_SRC,
            });
            const writeMapping = source.mapAsync(GPUMapMode.WRITE);
            const pendingState = source.mapState;
            await writeMapping;
            const mappedState = source.mapState;
            const lower = source.getMappedRange(0, 8);
            let overlapError = "";
            try { source.getMappedRange(4, 4); } catch (error) { overlapError = error.name; }
            const upper = source.getMappedRange(8, 8);
            new Uint32Array(lower).set([1, 2]);
            new Uint32Array(upper).set([3, 4]);
            source.unmap();
            const writeViewsDetached = lower.byteLength === 0 && upper.byteLength === 0;

            const readback = device.createBuffer({
                size: 16,
                usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
            });
            const encoder = device.createCommandEncoder();
            encoder.copyBufferToBuffer(source, 0, readback, 0, 16);
            device.queue.submit([encoder.finish()]);
            await readback.mapAsync(GPUMapMode.READ, 8, 8);
            let defaultRangeError = "";
            try { readback.getMappedRange(); } catch (error) { defaultRangeError = error.name; }
            const firstRead = readback.getMappedRange(8, 8);
            const copied = [...new Uint32Array(firstRead)];
            new Uint32Array(firstRead).fill(99);
            readback.unmap();
            const readViewDetached = firstRead.byteLength === 0;

            await readback.mapAsync(GPUMapMode.READ, 8, 8);
            const secondRead = [...new Uint32Array(readback.getMappedRange(8, 8))];
            readback.unmap();

            const invalid = device.createBuffer({ size: 4, usage: GPUBufferUsage.COPY_SRC });
            let invalidUsage = "";
            try { await invalid.mapAsync(GPUMapMode.WRITE); } catch (error) { invalidUsage = error.name; }
            const cancelled = device.createBuffer({
                size: 4,
                usage: GPUBufferUsage.MAP_WRITE | GPUBufferUsage.COPY_SRC,
            });
            const cancelledMapping = cancelled.mapAsync(GPUMapMode.WRITE);
            cancelled.unmap();
            let cancelledError = "";
            try { await cancelledMapping; } catch (error) { cancelledError = error.name; }
            source.destroy();
            gpuMapWriteResult = JSON.stringify({
                mappedState,
                pendingState,
                overlapError,
                defaultRangeError,
                copied,
                secondRead,
                writeViewsDetached,
                readViewDetached,
                invalidUsage,
                cancelledError,
                cancelledState: cancelled.mapState,
                destroyedState: source.mapState,
                native: [
                    GPUBuffer.prototype.mapAsync.toString(),
                    Object.getOwnPropertyDescriptor(GPUBuffer.prototype, "mapState").get.toString(),
                ],
            });
        }).catch(error => gpuMapWriteResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page.eval("gpuMapWriteResult").unwrap().to_string().unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"mappedState":"mapped","pendingState":"pending","overlapError":"OperationError","defaultRangeError":"OperationError","copied":[3,4],"secondRead":[3,4],"writeViewsDetached":true,"readViewDetached":true,"invalidUsage":"OperationError","cancelledError":"AbortError","cancelledState":"unmapped","destroyedState":"unmapped","native":["function mapAsync() { [native code] }","function get mapState() { [native code] }"]}"#,
        "unexpected WebGPU mapped-write result: {result}",
    );
}

#[test]
fn webgpu_render_bundles_are_reusable_and_reset_render_pass_state() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuRenderBundleResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuRenderBundleResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const shader = device.createShaderModule({ code: `
                    @vertex fn vertexMain(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                        const positions = array(
                            vec2f(-1, -1),
                            vec2f(3, -1),
                            vec2f(-1, 3),
                        );
                        return vec4f(positions[index], 0, 1);
                    }
                    @fragment fn fragmentMain() -> @location(0) vec4f {
                        return vec4f(0, 1, 0, 1);
                    }
                ` });
                const pipeline = device.createRenderPipeline({
                    layout: "auto",
                    vertex: { module: shader, entryPoint: "vertexMain" },
                    fragment: {
                        module: shader,
                        entryPoint: "fragmentMain",
                        targets: [{ format: "rgba8unorm" }],
                    },
                });
                const bundleEncoder = device.createRenderBundleEncoder({
                    colorFormats: ["rgba8unorm"],
                    label: "bundle encoder",
                });
                bundleEncoder.setPipeline(pipeline);
                bundleEncoder.draw(3);
                const bundle = bundleEncoder.finish({ label: "green bundle" });
                let secondFinish;
                try {
                    bundleEncoder.finish();
                    secondFinish = "none";
                } catch (error) {
                    secondFinish = error.name;
                }

                const textureUsage = GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC;
                const firstTexture = device.createTexture({
                    size: [1, 1], format: "rgba8unorm", usage: textureUsage,
                });
                const secondTexture = device.createTexture({
                    size: [1, 1], format: "rgba8unorm", usage: textureUsage,
                });
                const readback = device.createBuffer({
                    size: 512,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const commandEncoder = device.createCommandEncoder();
                const firstPass = commandEncoder.beginRenderPass({ colorAttachments: [{
                    view: firstTexture.createView(),
                    loadOp: "clear",
                    clearValue: [1, 0, 0, 1],
                    storeOp: "store",
                }] });
                firstPass.executeBundles([bundle]);
                let stateReset;
                try {
                    firstPass.draw(3);
                    stateReset = false;
                } catch (error) {
                    stateReset = error.name === "InvalidStateError";
                }
                firstPass.end();

                const secondPass = commandEncoder.beginRenderPass({ colorAttachments: [{
                    view: secondTexture.createView(),
                    loadOp: "clear",
                    clearValue: [0, 0, 1, 1],
                    storeOp: "store",
                }] });
                secondPass.executeBundles([bundle]);
                secondPass.end();
                commandEncoder.copyTextureToBuffer(
                    { texture: firstTexture },
                    { buffer: readback, offset: 0, bytesPerRow: 256 },
                    [1, 1],
                );
                commandEncoder.copyTextureToBuffer(
                    { texture: secondTexture },
                    { buffer: readback, offset: 256, bytesPerRow: 256 },
                    [1, 1],
                );
                device.queue.submit([commandEncoder.finish()]);
                return readback.mapAsync(GPUMapMode.READ).then(() => {
                    const bytes = new Uint8Array(readback.getMappedRange());
                    gpuRenderBundleResult = JSON.stringify({
                        encoder: bundleEncoder instanceof GPURenderBundleEncoder,
                        bundle: bundle instanceof GPURenderBundle,
                        encoderLabel: bundleEncoder.label,
                        bundleLabel: bundle.label,
                        secondFinish,
                        stateReset,
                        first: [...bytes.slice(0, 4)],
                        second: [...bytes.slice(256, 260)],
                        nativeCreate: GPUDevice.prototype.createRenderBundleEncoder.toString(),
                        nativeFinish: GPURenderBundleEncoder.prototype.finish.toString(),
                        nativeExecute: GPURenderPassEncoder.prototype.executeBundles.toString(),
                    });
                    readback.unmap();
                });
            });
        }).catch(error => gpuRenderBundleResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuRenderBundleResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"encoder":true,"bundle":true,"encoderLabel":"bundle encoder","bundleLabel":"green bundle","secondFinish":"InvalidStateError","stateReset":true,"first":[0,255,0,255],"second":[0,255,0,255],"nativeCreate":"function createRenderBundleEncoder() { [native code] }","nativeFinish":"function finish() { [native code] }","nativeExecute":"function executeBundles() { [native code] }"}"#,
        "unexpected WebGPU render-bundle result: {result}",
    );
}

#[test]
fn webgpu_debug_markers_encode_on_command_compute_and_render_encoders() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuDebugMarkerResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuDebugMarkerResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                device.pushErrorScope("validation");
                const encoder = device.createCommandEncoder();
                encoder.pushDebugGroup("command group");
                encoder.insertDebugMarker("command marker");
                encoder.popDebugGroup();

                const computePass = encoder.beginComputePass();
                computePass.pushDebugGroup("compute group");
                computePass.insertDebugMarker("compute marker");
                computePass.popDebugGroup();
                computePass.end();
                let computeEnded;
                try {
                    computePass.insertDebugMarker("late compute marker");
                    computeEnded = false;
                } catch (error) {
                    computeEnded = error.name === "InvalidStateError";
                }

                const texture = device.createTexture({
                    size: [1, 1],
                    format: "rgba8unorm",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT,
                });
                const renderPass = encoder.beginRenderPass({ colorAttachments: [{
                    view: texture.createView(),
                    loadOp: "clear",
                    clearValue: [0, 0, 0, 1],
                    storeOp: "store",
                }] });
                renderPass.pushDebugGroup("render group");
                renderPass.insertDebugMarker("render marker");
                renderPass.popDebugGroup();
                renderPass.end();
                let renderEnded;
                try {
                    renderPass.popDebugGroup();
                    renderEnded = false;
                } catch (error) {
                    renderEnded = error.name === "InvalidStateError";
                }

                const command = encoder.finish();
                let encoderEnded;
                try {
                    encoder.pushDebugGroup("late command group");
                    encoderEnded = false;
                } catch (error) {
                    encoderEnded = error.name === "InvalidStateError";
                }
                device.queue.submit([command]);
                return device.popErrorScope().then(error => {
                    gpuDebugMarkerResult = JSON.stringify({
                        validationError: error === null,
                        computeEnded,
                        renderEnded,
                        encoderEnded,
                        bundleSurface: typeof GPURenderBundleEncoder.prototype.insertDebugMarker,
                        native: [
                            GPUCommandEncoder.prototype.insertDebugMarker.toString(),
                            GPUCommandEncoder.prototype.pushDebugGroup.toString(),
                            GPUCommandEncoder.prototype.popDebugGroup.toString(),
                            GPUComputePassEncoder.prototype.insertDebugMarker.toString(),
                            GPUComputePassEncoder.prototype.pushDebugGroup.toString(),
                            GPUComputePassEncoder.prototype.popDebugGroup.toString(),
                            GPURenderPassEncoder.prototype.insertDebugMarker.toString(),
                            GPURenderPassEncoder.prototype.pushDebugGroup.toString(),
                            GPURenderPassEncoder.prototype.popDebugGroup.toString(),
                        ],
                    });
                });
            });
        }).catch(error => gpuDebugMarkerResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuDebugMarkerResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"validationError":true,"computeEnded":true,"renderEnded":true,"encoderEnded":true,"bundleSurface":"undefined","native":["function insertDebugMarker() { [native code] }","function pushDebugGroup() { [native code] }","function popDebugGroup() { [native code] }","function insertDebugMarker() { [native code] }","function pushDebugGroup() { [native code] }","function popDebugGroup() { [native code] }","function insertDebugMarker() { [native code] }","function pushDebugGroup() { [native code] }","function popDebugGroup() { [native code] }"]}"#,
        "unexpected WebGPU debug-marker result: {result}",
    );
}

#[test]
fn webgpu_immediate_data_is_negotiated_and_executes_in_compute_render_and_bundles() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuImmediateResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuImmediateResult = "no-adapter"; return; }
            if (!adapter.features.has("immediates")) {
                gpuImmediateResult = `unsupported:${adapter.limits.maxImmediateSize}`;
                return;
            }
            return adapter.requestDevice({
                requiredFeatures: ["immediates"],
                requiredLimits: { maxImmediateSize: 4 },
            }).then(device => {
                const storageLayout = device.createBindGroupLayout({ entries: [{
                    binding: 0,
                    visibility: GPUShaderStage.COMPUTE,
                    buffer: { type: "storage" },
                }] });
                const computeLayout = device.createPipelineLayout({
                    bindGroupLayouts: [storageLayout],
                    immediateSize: 4,
                });
                const renderLayout = device.createPipelineLayout({
                    bindGroupLayouts: [],
                    immediateSize: 4,
                });
                const computeShader = device.createShaderModule({ code: `
                    var<immediate> immediateValue: u32;
                    @group(0) @binding(0) var<storage, read_write> output: array<u32>;

                    @compute @workgroup_size(1)
                    fn computeMain() {
                        output[0] = immediateValue;
                    }
                ` });
                const renderShader = device.createShaderModule({ code: `
                    var<immediate> immediateValue: u32;

                    @vertex fn vertexMain(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                        const positions = array(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
                        return vec4f(positions[index], 0, 1);
                    }

                    @fragment fn fragmentMain() -> @location(0) vec4f {
                        if (immediateValue == 7u) { return vec4f(0, 1, 0, 1); }
                        return vec4f(0, 0, 1, 1);
                    }
                ` });
                const computePipeline = device.createComputePipeline({
                    layout: computeLayout,
                    compute: { module: computeShader, entryPoint: "computeMain" },
                });
                device.pushErrorScope("validation");
                const renderPipeline = device.createRenderPipeline({
                    layout: renderLayout,
                    vertex: { module: renderShader, entryPoint: "vertexMain" },
                    fragment: {
                        module: renderShader,
                        entryPoint: "fragmentMain",
                        targets: [{ format: "rgba8unorm" }],
                    },
                });
                return device.popErrorScope().then(pipelineError => {
                if (pipelineError !== null) throw new Error(`render pipeline: ${pipelineError.message}`);
                const output = device.createBuffer({
                    size: 4,
                    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
                });
                const group = device.createBindGroup({
                    layout: storageLayout,
                    entries: [{ binding: 0, resource: { buffer: output } }],
                });
                const textureUsage = GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC;
                const directTexture = device.createTexture({
                    size: [1, 1], format: "rgba8unorm", usage: textureUsage,
                });
                const bundleTexture = device.createTexture({
                    size: [1, 1], format: "rgba8unorm", usage: textureUsage,
                });
                const readback = device.createBuffer({
                    size: 768,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });

                const bundleEncoder = device.createRenderBundleEncoder({
                    colorFormats: ["rgba8unorm"],
                });
                bundleEncoder.setPipeline(renderPipeline);
                bundleEncoder.setImmediates(0, new Uint32Array([9]));
                bundleEncoder.draw(3);
                const bundle = bundleEncoder.finish();

                const encoder = device.createCommandEncoder();
                const computePass = encoder.beginComputePass();
                computePass.setPipeline(computePipeline);
                computePass.setBindGroup(0, group);
                computePass.setImmediates(0, new Uint32Array([99, 123, 99]), 1, 1);
                computePass.dispatchWorkgroups(1);
                computePass.end();

                const directPass = encoder.beginRenderPass({ colorAttachments: [{
                    view: directTexture.createView(), loadOp: "clear",
                    clearValue: [1, 0, 0, 1], storeOp: "store",
                }] });
                directPass.setPipeline(renderPipeline);
                directPass.setImmediates(0, new Uint32Array([99, 7, 99]), 1, 1);
                directPass.draw(3);
                directPass.end();

                const bundlePass = encoder.beginRenderPass({ colorAttachments: [{
                    view: bundleTexture.createView(), loadOp: "clear",
                    clearValue: [1, 0, 0, 1], storeOp: "store",
                }] });
                bundlePass.executeBundles([bundle]);
                bundlePass.end();
                encoder.copyTextureToBuffer(
                    { texture: directTexture },
                    { buffer: readback, offset: 0, bytesPerRow: 256 },
                    [1, 1],
                );
                encoder.copyTextureToBuffer(
                    { texture: bundleTexture },
                    { buffer: readback, offset: 256, bytesPerRow: 256 },
                    [1, 1],
                );
                encoder.copyBufferToBuffer(output, 0, readback, 512, 4);
                device.queue.submit([encoder.finish()]);
                return readback.mapAsync(GPUMapMode.READ).then(() => {
                    const bytes = new Uint8Array(readback.getMappedRange());
                    gpuImmediateResult = JSON.stringify({
                        adapterLimit: adapter.limits.maxImmediateSize >= 4,
                        deviceLimit: device.limits.maxImmediateSize >= 4,
                        feature: device.features.has("immediates"),
                        compute: new Uint32Array(bytes.buffer, bytes.byteOffset + 512, 1)[0],
                        direct: [...bytes.slice(0, 4)],
                        bundle: [...bytes.slice(256, 260)],
                        native: [
                            GPUComputePassEncoder.prototype.setImmediates.toString(),
                            GPURenderPassEncoder.prototype.setImmediates.toString(),
                            GPURenderBundleEncoder.prototype.setImmediates.toString(),
                        ],
                    });
                    readback.unmap();
                });
                });
            });
        }).catch(error => gpuImmediateResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuImmediateResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result == "unsupported:0"
            || result
                == r#"{"adapterLimit":true,"deviceLimit":true,"feature":true,"compute":123,"direct":[0,255,0,255],"bundle":[0,0,255,255],"native":["function setImmediates() { [native code] }","function setImmediates() { [native code] }","function setImmediates() { [native code] }"]}"#,
        "unexpected WebGPU immediate-data result: {result}",
    );
}

#[test]
fn webgpu_capabilities_follow_the_page_persona_and_readonly_webidl_shapes() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuCapabilityResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            const globalShape = {
                wgslBrand: navigator.gpu.wgslLanguageFeatures instanceof WGSLLanguageFeatures,
                wgslReadonly: typeof navigator.gpu.wgslLanguageFeatures.add === "undefined",
                preferredFormat: navigator.gpu.getPreferredCanvasFormat(),
                settersHidden: !("__brimpSetGpuPersona" in globalThis)
                    && !("__brimpSetWebGlPersona" in globalThis),
            };
            if (!adapter) {
                gpuCapabilityResult = JSON.stringify({ adapter: false, globalShape });
                return;
            }
            adapter.requestDevice().then(device => {
                const info = adapter.info;
                gpuCapabilityResult = JSON.stringify({
                    adapter: true,
                    globalShape,
                    adapterBrand: adapter.features instanceof GPUSupportedFeatures,
                    deviceBrand: device.features instanceof GPUSupportedFeatures,
                    readonly: typeof adapter.features.add === "undefined"
                        && typeof adapter.features.delete === "undefined"
                        && typeof adapter.features.clear === "undefined",
                    iterable: [...adapter.features].every(value => adapter.features.has(value)),
                    prototype: Object.getPrototypeOf(adapter.features) === GPUSupportedFeatures.prototype
                        && Object.getPrototypeOf(GPUSupportedFeatures.prototype) === Object.prototype,
                    info: { ...info },
                    infoKeys: Object.keys(info),
                    noDriverExtras: !("backend" in info) && !("deviceType" in info),
                    adapterLimit: adapter.limits.maxBindGroupsPlusVertexBuffers,
                    deviceLimit: device.limits.maxBindGroupsPlusVertexBuffers,
                    native: [
                        GPUSupportedFeatures.prototype.has.toString(),
                        Object.getOwnPropertyDescriptor(GPUSupportedFeatures.prototype, "size").get.toString(),
                        WGSLLanguageFeatures.toString(),
                    ],
                });
            });
        }).catch(error => gpuCapabilityResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuCapabilityResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        !result.starts_with("error:"),
        "unexpected WebGPU error: {result}"
    );
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["globalShape"]["wgslBrand"], true);
    assert_eq!(result["globalShape"]["wgslReadonly"], true);
    assert_eq!(result["globalShape"]["preferredFormat"], "bgra8unorm");
    assert_eq!(result["globalShape"]["settersHidden"], true);
    if result["adapter"] == true {
        assert_eq!(result["adapterBrand"], true);
        assert_eq!(result["deviceBrand"], true);
        assert_eq!(result["readonly"], true);
        assert_eq!(result["iterable"], true);
        assert_eq!(result["prototype"], true);
        assert_eq!(
            result["info"],
            serde_json::json!({
                "vendor": "",
                "architecture": "",
                "device": "",
                "description": "",
            })
        );
        assert_eq!(
            result["infoKeys"],
            serde_json::json!(["vendor", "architecture", "device", "description"])
        );
        assert_eq!(result["noDriverExtras"], true);
        let adapter_limit = result["adapterLimit"].as_u64().unwrap();
        let device_limit = result["deviceLimit"].as_u64().unwrap();
        assert!(adapter_limit > 0 && adapter_limit <= 24);
        assert!(device_limit > 0 && device_limit <= adapter_limit);
        assert_eq!(
            result["native"],
            serde_json::json!([
                "function has() { [native code] }",
                "function get size() { [native code] }",
                "function WGSLLanguageFeatures() { [native code] }",
            ])
        );
    }
}

#[test]
fn webgpu_required_limits_are_negotiated_and_rejected_coherently() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuRequiredLimitsResult = "pending";
        navigator.gpu.requestAdapter().then(async adapter => {
            if (!adapter) { gpuRequiredLimitsResult = "no-adapter"; return; }
            const rejection = promise => promise.then(
                () => "resolved",
                error => `${error.name}:${error.message}`,
            );
            const device = await adapter.requestDevice({
                label: "required-limits-device",
                requiredLimits: {
                    maxBufferSize: adapter.limits.maxBufferSize,
                    minUniformBufferOffsetAlignment: adapter.limits.minUniformBufferOffsetAlignment,
                },
            });
            const [tooLarge, tooWellAligned, unknown, fractional] = await Promise.all([
                rejection(adapter.requestDevice({
                    requiredLimits: { maxBufferSize: adapter.limits.maxBufferSize + 1 },
                })),
                rejection(adapter.requestDevice({
                    requiredLimits: {
                        minUniformBufferOffsetAlignment:
                            adapter.limits.minUniformBufferOffsetAlignment - 1,
                    },
                })),
                rejection(adapter.requestDevice({ requiredLimits: { imaginaryLimit: 1 } })),
                rejection(adapter.requestDevice({ requiredLimits: { maxBufferSize: 1.5 } })),
            ]);
            gpuRequiredLimitsResult = JSON.stringify({
                label: device.label,
                maxBufferSize: device.limits.maxBufferSize,
                requestedMaxBufferSize: adapter.limits.maxBufferSize,
                minUniformBufferOffsetAlignment: device.limits.minUniformBufferOffsetAlignment,
                requestedMinUniformBufferOffsetAlignment:
                    adapter.limits.minUniformBufferOffsetAlignment,
                tooLarge,
                tooWellAligned,
                unknown,
                fractional,
            });
        }).catch(error => gpuRequiredLimitsResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuRequiredLimitsResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter" || !result.starts_with("error:"),
        "unexpected WebGPU required-limits error: {result}",
    );
    if result != "no-adapter" {
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(result["label"], "required-limits-device");
        assert_eq!(result["maxBufferSize"], result["requestedMaxBufferSize"]);
        assert_eq!(
            result["minUniformBufferOffsetAlignment"],
            result["requestedMinUniformBufferOffsetAlignment"]
        );
        assert!(
            result["tooLarge"]
                .as_str()
                .unwrap()
                .starts_with("OperationError:")
        );
        assert!(
            result["tooWellAligned"]
                .as_str()
                .unwrap()
                .starts_with("OperationError:")
        );
        assert!(
            result["unknown"]
                .as_str()
                .unwrap()
                .starts_with("OperationError:")
        );
        assert!(
            result["fractional"]
                .as_str()
                .unwrap()
                .starts_with("TypeError:")
        );
    }
}

#[test]
fn webgpu_optional_features_reflect_and_negotiate_native_support() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuOptionalFeaturesResult = "pending";
        navigator.gpu.requestAdapter().then(async adapter => {
            if (!adapter) { gpuOptionalFeaturesResult = "no-adapter"; return; }
            const recognized = new Set([
                "depth-clip-control",
                "depth32float-stencil8",
                "texture-compression-bc",
                "texture-compression-bc-sliced-3d",
                "texture-compression-etc2",
                "texture-compression-astc",
                "texture-compression-astc-sliced-3d",
                "timestamp-query",
                "indirect-first-instance",
                "shader-f16",
                "rg11b10ufloat-renderable",
                "bgra8unorm-storage",
                "float32-filterable",
                "float32-blendable",
                "clip-distances",
                "dual-source-blending",
                "primitive-index",
                "immediates",
            ]);
            const supported = [...adapter.features];
            const device = await adapter.requestDevice({ requiredFeatures: supported });
            const textures = [
                device.createTexture({
                    size: [4, 4],
                    format: "r32float",
                    usage: GPUTextureUsage.TEXTURE_BINDING,
                }),
                device.createTexture({
                    size: [4, 4],
                    format: "rgba32float",
                    usage: GPUTextureUsage.TEXTURE_BINDING,
                }),
            ];
            if (supported.includes("depth32float-stencil8")) textures.push(device.createTexture({
                size: [4, 4],
                format: "depth32float-stencil8",
                usage: GPUTextureUsage.RENDER_ATTACHMENT,
            }));
            if (supported.includes("rg11b10ufloat-renderable")) textures.push(device.createTexture({
                size: [4, 4],
                format: "rg11b10ufloat",
                usage: GPUTextureUsage.RENDER_ATTACHMENT,
            }));
            if (supported.includes("bgra8unorm-storage")) textures.push(device.createTexture({
                size: [4, 4],
                format: "bgra8unorm",
                usage: GPUTextureUsage.STORAGE_BINDING,
            }));
            if (supported.includes("float32-blendable")) textures.push(device.createTexture({
                size: [4, 4],
                format: "rgba32float",
                usage: GPUTextureUsage.RENDER_ATTACHMENT,
            }));
            if (supported.includes("texture-compression-bc")) textures.push(device.createTexture({
                size: [4, 4],
                format: "bc1-rgba-unorm",
                usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
            }));
            if (supported.includes("texture-compression-etc2")) textures.push(device.createTexture({
                size: [4, 4],
                format: "etc2-rgb8unorm",
                usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
            }));
            if (supported.includes("texture-compression-astc")) textures.push(device.createTexture({
                size: [4, 4],
                format: "astc-4x4-unorm",
                usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
            }));
            const module = device.createShaderModule({ code: `
                @vertex fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                    var positions = array(vec2f(-1, -1), vec2f(1, -1), vec2f(0, 1));
                    return vec4f(positions[index], 0, 1);
                }
                @fragment fn fragment_main() -> @location(0) vec4f {
                    return vec4f(1, 0, 0, 1);
                }
            ` });
            const pipeline = device.createRenderPipeline({
                layout: "auto",
                vertex: { module, entryPoint: "vertex_main" },
                fragment: {
                    module,
                    entryPoint: "fragment_main",
                    targets: [{ format: "rgba8unorm" }],
                },
                primitive: {
                    frontFace: "cw",
                    unclippedDepth: supported.includes("depth-clip-control"),
                },
            });
            const sliced = supported.includes("texture-compression-bc-sliced-3d")
                ? ["texture-compression-bc-sliced-3d", "texture-compression-bc"]
                : supported.includes("texture-compression-astc-sliced-3d")
                    ? ["texture-compression-astc-sliced-3d", "texture-compression-astc"]
                    : null;
            let dependency = true;
            if (sliced) {
                const dependencyDevice = await adapter.requestDevice({ requiredFeatures: [sliced[0]] });
                dependency = sliced.every(feature => dependencyDevice.features.has(feature));
            }
            const unknown = await adapter.requestDevice({ requiredFeatures: ["imaginary-feature"] })
                .then(() => "resolved", error => `${error.name}:${error.message}`);
            gpuOptionalFeaturesResult = JSON.stringify({
                recognized: supported.every(feature => recognized.has(feature)),
                negotiated: JSON.stringify([...device.features]) === JSON.stringify(supported),
                formats: textures.every(texture => texture instanceof GPUTexture),
                pipeline: pipeline instanceof GPURenderPipeline,
                dependency,
                unknown,
            });
        }).catch(error => gpuOptionalFeaturesResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuOptionalFeaturesResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter" || !result.starts_with("error:"),
        "unexpected WebGPU optional-feature error: {result}",
    );
    if result != "no-adapter" {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&result).unwrap(),
            serde_json::json!({
                "recognized": true,
                "negotiated": true,
                "formats": true,
                "pipeline": true,
                "dependency": true,
                "unknown": "NotSupportedError:A required GPU feature is unavailable",
            }),
        );
    }
}

#[test]
fn webgpu_texture_views_select_formats_dimensions_and_subresources() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuTextureViewResult = "pending";
        navigator.gpu.requestAdapter().then(async adapter => {
            if (!adapter) { gpuTextureViewResult = "no-adapter"; return; }
            const device = await adapter.requestDevice();
            device.pushErrorScope("validation");
            const texture = device.createTexture({
                label: "view-source",
                size: [8, 8, 6],
                mipLevelCount: 3,
                format: "rgba8unorm",
                viewFormats: ["rgba8unorm-srgb"],
                usage: GPUTextureUsage.COPY_SRC
                    | GPUTextureUsage.TEXTURE_BINDING
                    | GPUTextureUsage.RENDER_ATTACHMENT,
            });
            const layer = texture.createView({
                label: "mip-layer",
                format: "rgba8unorm-srgb",
                dimension: "2d",
                usage: GPUTextureUsage.RENDER_ATTACHMENT,
                baseMipLevel: 1,
                mipLevelCount: 1,
                baseArrayLayer: 2,
                arrayLayerCount: 1,
            });
            const array = texture.createView({
                dimension: "2d-array",
                baseArrayLayer: 1,
                arrayLayerCount: 4,
            });
            const cube = texture.createView({ dimension: "cube", arrayLayerCount: 6 });
            const depth = device.createTexture({
                size: [4, 4],
                format: "depth24plus-stencil8",
                usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING,
            });
            const depthOnly = depth.createView({ aspect: "depth-only" });
            const stencilOnly = depth.createView({ aspect: "stencil-only" });
            const readback = device.createBuffer({
                size: 1024,
                usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
            });
            const encoder = device.createCommandEncoder();
            const emptyComputePass = encoder.beginComputePass();
            emptyComputePass.end();
            const pass = encoder.beginRenderPass({ colorAttachments: [{
                view: layer,
                loadOp: "clear",
                clearValue: { r: 0, g: 1, b: 0, a: 1 },
                storeOp: "store",
            }] });
            pass.end();
            encoder.copyTextureToBuffer(
                { texture, mipLevel: 1, origin: [0, 0, 2] },
                { buffer: readback, bytesPerRow: 256, rowsPerImage: 4 },
                [4, 4, 1],
            );
            device.queue.submit([encoder.finish()]);
            await readback.mapAsync(GPUMapMode.READ);
            const pixel = [...new Uint8Array(readback.getMappedRange()).slice(0, 4)];
            readback.unmap();
            const error = await device.popErrorScope();
            gpuTextureViewResult = JSON.stringify({
                views: [layer, array, cube, depthOnly, stencilOnly]
                    .every(view => view instanceof GPUTextureView),
                textureLabel: texture.label,
                viewLabel: layer.label,
                pixel,
                error: error === null,
            });
        }).catch(error => gpuTextureViewResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuTextureViewResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter" || !result.starts_with("error:"),
        "unexpected WebGPU texture-view error: {result}",
    );
    if result != "no-adapter" {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&result).unwrap(),
            serde_json::json!({
                "views": true,
                "textureLabel": "view-source",
                "viewLabel": "mip-layer",
                "pixel": [0, 255, 0, 255],
                "error": true,
            }),
        );
    }
}

#[test]
fn webgpu_external_image_copies_crop_flip_and_convert_canvas_sources() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().canvas(true).webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuExternalImageResult = "pending";
        navigator.gpu.requestAdapter().then(async adapter => {
            if (!adapter) { gpuExternalImageResult = "no-adapter"; return; }
            const device = await adapter.requestDevice();
            const canvas = document.createElement("canvas");
            canvas.width = 3;
            canvas.height = 2;
            const context = canvas.getContext("2d");
            context.putImageData(new ImageData(new Uint8ClampedArray([
                1, 2, 3, 255, 10, 20, 30, 255, 100, 50, 20, 128,
                4, 5, 6, 255, 40, 50, 60, 255, 80, 40, 20, 128,
            ]), 3, 2), 0, 0);
            const bitmap = await createImageBitmap(canvas);
            const rgba = device.createTexture({
                size: [2, 2],
                format: "rgba8unorm",
                usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC,
            });
            const bgra = device.createTexture({
                size: [3, 2],
                format: "bgra8unorm",
                usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC,
            });
            const wide = device.createTexture({
                size: [2, 1],
                format: "rgba8unorm",
                usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC,
            });
            const p3Canvas = document.createElement("canvas");
            p3Canvas.width = p3Canvas.height = 1;
            const p3Context = p3Canvas.getContext("2d", {
                colorSpace: "display-p3",
                colorType: "float16",
            });
            p3Context.putImageData(new ImageData(
                new Uint8ClampedArray([255, 128, 0, 255]),
                1,
                1,
                { colorSpace: "display-p3" },
            ), 0, 0);
            device.queue.copyExternalImageToTexture(
                { source: p3Canvas },
                { texture: wide, colorSpace: "srgb" },
                [1, 1],
            );
            device.queue.copyExternalImageToTexture(
                { source: new ImageData(
                    new Float16Array([0.25, 0.5, 1.25, 1]),
                    1,
                    1,
                    { colorSpace: "display-p3", pixelFormat: "rgba-float16" },
                ) },
                { texture: wide, origin: [1, 0], colorSpace: "display-p3" },
                [1, 1],
            );
            device.pushErrorScope("validation");
            device.queue.copyExternalImageToTexture(
                { source: canvas, origin: [1, 0], flipY: true },
                { texture: rgba, premultipliedAlpha: true },
                [2, 2],
            );
            device.queue.copyExternalImageToTexture(
                { source: bitmap },
                { texture: bgra },
                [3, 2],
            );
            const rgbaReadback = device.createBuffer({
                size: 512,
                usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
            });
            const bgraReadback = device.createBuffer({
                size: 512,
                usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
            });
            const wideReadback = device.createBuffer({
                size: 256,
                usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
            });
            const encoder = device.createCommandEncoder();
            encoder.copyTextureToBuffer(
                { texture: rgba },
                { buffer: rgbaReadback, bytesPerRow: 256, rowsPerImage: 2 },
                [2, 2, 1],
            );
            encoder.copyTextureToBuffer(
                { texture: bgra },
                { buffer: bgraReadback, bytesPerRow: 256, rowsPerImage: 2 },
                [3, 2, 1],
            );
            encoder.copyTextureToBuffer(
                { texture: wide },
                { buffer: wideReadback, bytesPerRow: 256 },
                [2, 1, 1],
            );
            device.queue.submit([encoder.finish()]);
            await Promise.all([
                rgbaReadback.mapAsync(GPUMapMode.READ),
                bgraReadback.mapAsync(GPUMapMode.READ),
                wideReadback.mapAsync(GPUMapMode.READ),
            ]);
            const rgbaBytes = new Uint8Array(rgbaReadback.getMappedRange());
            const bgraBytes = new Uint8Array(bgraReadback.getMappedRange());
            const widePixels = [...new Uint8Array(wideReadback.getMappedRange()).slice(0, 8)];
            const rgbaPixels = [
                ...rgbaBytes.slice(0, 8),
                ...rgbaBytes.slice(256, 264),
            ];
            const bgraPixels = [
                ...bgraBytes.slice(0, 12),
                ...bgraBytes.slice(256, 268),
            ];
            rgbaReadback.unmap();
            bgraReadback.unmap();
            wideReadback.unmap();
            const error = await device.popErrorScope();
            const colorSpace = (() => {
                try {
                    device.queue.copyExternalImageToTexture(
                        { source: canvas },
                        { texture: rgba, colorSpace: "display-p3" },
                        [2, 2],
                    );
                    return "none";
                } catch (error) { return error.name; }
            })();
            const invalidColorSpace = (() => {
                try {
                    device.queue.copyExternalImageToTexture(
                        { source: canvas },
                        { texture: rgba, colorSpace: "rec2020" },
                        [2, 2],
                    );
                    return "none";
                } catch (error) { return error.name; }
            })();
            bitmap.close();
            gpuExternalImageResult = JSON.stringify({
                rgbaPixels,
                bgraPixels,
                widePixels,
                error: error === null,
                colorSpace,
                invalidColorSpace,
                native: GPUQueue.prototype.copyExternalImageToTexture.toString(),
            });
        }).catch(error => gpuExternalImageResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuExternalImageResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter" || !result.starts_with("error:"),
        "unexpected WebGPU external-image error: {result}",
    );
    if result != "no-adapter" {
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            result["rgbaPixels"],
            serde_json::json!([
                40, 50, 60, 255, 40, 20, 10, 128, 10, 20, 30, 255, 50, 25, 10, 128
            ])
        );
        assert_eq!(
            result["bgraPixels"],
            serde_json::json!([
                3, 2, 1, 255, 30, 20, 10, 255, 20, 50, 100, 128, 6, 5, 4, 255, 60, 50, 40, 255, 20,
                40, 80, 128
            ])
        );
        assert_eq!(result["widePixels"][0], 255);
        assert!(result["widePixels"][1].as_u64().unwrap() > 110);
        assert!(result["widePixels"][1].as_u64().unwrap() < 126);
        assert_eq!(result["widePixels"][2], 0);
        assert_eq!(result["widePixels"][3], 255);
        assert_eq!(
            &result["widePixels"].as_array().unwrap()[4..],
            &serde_json::json!([64, 128, 255, 255]).as_array().unwrap()[..]
        );
        assert_eq!(result["error"], true);
        assert_eq!(result["colorSpace"], "none");
        assert_eq!(result["invalidColorSpace"], "TypeError");
        assert_eq!(
            result["native"],
            "function copyExternalImageToTexture() { [native code] }"
        );
    }
}

#[test]
fn webgpu_reports_native_shader_diagnostics_and_waits_for_submitted_work() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuDiagnostics = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuDiagnostics = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const valid = device.createShaderModule({ code: `
                    @compute @workgroup_size(1)
                    fn main() {}
                ` });
                const invalid = device.createShaderModule({ code: `
                    // 😀 UTF-16 location probe
                    @compute @workgroup_size(1)
                    fn main( {
                ` });
                const source = device.createBuffer({
                    size: 4,
                    usage: GPUBufferUsage.COPY_SRC,
                    mappedAtCreation: true,
                });
                new Uint32Array(source.getMappedRange())[0] = 0x12345678;
                source.unmap();
                const destination = device.createBuffer({
                    size: 4,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const encoder = device.createCommandEncoder();
                encoder.copyBufferToBuffer(source, 0, destination, 0, 4);
                device.queue.submit([encoder.finish()]);
                return Promise.all([
                    valid.getCompilationInfo(),
                    invalid.getCompilationInfo(),
                    device.queue.onSubmittedWorkDone(),
                ]).then(([validInfo, invalidInfo]) => {
                    return destination.mapAsync(GPUMapMode.READ).then(() => {
                        const copied = new Uint32Array(destination.getMappedRange())[0];
                        destination.unmap();
                        const message = invalidInfo.messages[0];
                        gpuDiagnostics = JSON.stringify({
                            validInfo: validInfo instanceof GPUCompilationInfo,
                            validMessages: validInfo.messages.length,
                            invalidInfo: invalidInfo instanceof GPUCompilationInfo,
                            message: message instanceof GPUCompilationMessage,
                            messageType: message?.type,
                            hasText: message?.message.length > 0,
                            hasLocation: message?.lineNum > 0 && message?.linePos > 0
                                && message?.offset >= 0 && message?.length > 0,
                            frozen: Object.isFrozen(invalidInfo) && Object.isFrozen(invalidInfo.messages)
                                && Object.isFrozen(message),
                            copied,
                            native: [invalid.getCompilationInfo.toString(), device.queue.onSubmittedWorkDone.toString()],
                        });
                    });
                });
            });
        }).catch(error => gpuDiagnostics = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page.eval("gpuDiagnostics").unwrap().to_string().unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"validInfo":true,"validMessages":0,"invalidInfo":true,"message":true,"messageType":"error","hasText":true,"hasLocation":true,"frozen":true,"copied":305419896,"native":["function getCompilationInfo() { [native code] }","function onSubmittedWorkDone() { [native code] }"]}"#,
        "unexpected WebGPU diagnostics result: {result}",
    );
}

#[test]
fn webgpu_encoder_clears_buffers_and_copies_between_textures() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuEncoderTransfers = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuEncoderTransfers = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const buffer = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
                    mappedAtCreation: true,
                });
                new Uint32Array(buffer.getMappedRange()).set([1, 2, 3, 4]);
                buffer.unmap();
                const bufferReadback = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });

                const sourceTexture = device.createTexture({
                    size: [2, 2], format: "rgba8unorm",
                    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
                });
                const destinationTexture = device.createTexture({
                    size: [2, 2], format: "rgba8unorm",
                    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST,
                });
                device.queue.writeTexture(
                    { texture: sourceTexture },
                    new Uint8Array([
                        255, 0, 0, 255, 0, 255, 0, 255,
                        0, 0, 255, 255, 255, 255, 255, 255,
                    ]),
                    { bytesPerRow: 8, rowsPerImage: 2 },
                    [2, 2],
                );
                const textureReadback = device.createBuffer({
                    size: 512,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });

                const encoder = device.createCommandEncoder();
                encoder.clearBuffer(buffer, 4, 8);
                encoder.copyBufferToBuffer(buffer, 0, bufferReadback, 0, 16);
                encoder.copyTextureToTexture(
                    { texture: sourceTexture, origin: [1, 0, 0], aspect: "all" },
                    { texture: destinationTexture, origin: [0, 0, 0], aspect: "all" },
                    [1, 2, 1],
                );
                encoder.copyTextureToBuffer(
                    { texture: destinationTexture },
                    { buffer: textureReadback, bytesPerRow: 256, rowsPerImage: 2 },
                    [2, 2, 1],
                );
                device.queue.submit([encoder.finish()]);
                return device.queue.onSubmittedWorkDone().then(() => Promise.all([
                    bufferReadback.mapAsync(GPUMapMode.READ),
                    textureReadback.mapAsync(GPUMapMode.READ),
                ])).then(() => {
                    const cleared = [...new Uint32Array(bufferReadback.getMappedRange())];
                    const textureBytes = new Uint8Array(textureReadback.getMappedRange());
                    const copied = [...textureBytes.slice(0, 8), ...textureBytes.slice(256, 264)];
                    bufferReadback.unmap();
                    textureReadback.unmap();
                    gpuEncoderTransfers = JSON.stringify({
                        cleared,
                        copied,
                        native: [encoder.clearBuffer.toString(), encoder.copyTextureToTexture.toString()],
                    });
                });
            });
        }).catch(error => gpuEncoderTransfers = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuEncoderTransfers")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"cleared":[1,0,0,4],"copied":[0,255,0,255,0,0,0,0,255,255,255,255,0,0,0,0],"native":["function clearBuffer() { [native code] }","function copyTextureToTexture() { [native code] }"]}"#,
        "unexpected WebGPU encoder transfer result: {result}",
    );
}

#[test]
fn webgpu_depth_and_stencil_state_controls_render_visibility() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuDepthResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuDepthResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const shader = device.createShaderModule({ code: `
                    struct Color { value: vec4f }
                    @group(0) @binding(0) var<uniform> color: Color;
                    @vertex fn vertexMain(@location(0) position: vec3f) -> @builtin(position) vec4f {
                        return vec4f(position, 1);
                    }
                    @fragment fn fragmentMain() -> @location(0) vec4f { return color.value; }
                ` });
                const bindGroupLayout = device.createBindGroupLayout({
                    entries: [{ binding: 0, visibility: GPUShaderStage.FRAGMENT, buffer: { type: "uniform", minBindingSize: 16 } }],
                });
                const pipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [bindGroupLayout] });
                const stencil = (compare, passOp = "keep") => ({ compare, failOp: "keep", depthFailOp: "keep", passOp });
                const pipeline = (stencilState) => device.createRenderPipeline({
                    layout: pipelineLayout,
                    vertex: { module: shader, entryPoint: "vertexMain", buffers: [{
                        arrayStride: 12,
                        attributes: [{ format: "float32x3", offset: 0, shaderLocation: 0 }],
                    }] },
                    fragment: { module: shader, entryPoint: "fragmentMain", targets: [{ format: "rgba8unorm" }] },
                    depthStencil: {
                        format: "depth24plus-stencil8",
                        depthWriteEnabled: true,
                        depthCompare: "less",
                        stencilFront: stencilState,
                        stencilBack: stencilState,
                        stencilReadMask: 0xff,
                        stencilWriteMask: 0xff,
                    },
                });
                const writePipeline = pipeline(stencil("always", "replace"));
                const stencilRejectPipeline = pipeline(stencil("not-equal"));
                const depthRejectPipeline = pipeline(stencil("always"));
                const vertices = device.createBuffer({
                    size: 108,
                    usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(vertices, 0, new Float32Array([
                    -1, -1, 0.5, 3, -1, 0.5, -1, 3, 0.5,
                    -1, -1, 0.25, 3, -1, 0.25, -1, 3, 0.25,
                    -1, -1, 0.75, 3, -1, 0.75, -1, 3, 0.75,
                ]));
                const colorBuffer = value => {
                    const buffer = device.createBuffer({ size: 16, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST });
                    device.queue.writeBuffer(buffer, 0, new Float32Array(value));
                    return buffer;
                };
                const green = colorBuffer([0, 1, 0, 1]);
                const red = colorBuffer([1, 0, 0, 1]);
                const group = buffer => device.createBindGroup({
                    layout: bindGroupLayout,
                    entries: [{ binding: 0, resource: { buffer } }],
                });
                const greenGroup = group(green);
                const stencilRejectGroup = group(red);
                const depthRejectGroup = group(red);
                const color = device.createTexture({
                    size: [4, 4], format: "rgba8unorm",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
                });
                const depthStencil = device.createTexture({
                    size: [4, 4], format: "depth24plus-stencil8",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT,
                });
                const readback = device.createBuffer({
                    size: 256 * 4,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const encoder = device.createCommandEncoder();
                const pass = encoder.beginRenderPass({
                    colorAttachments: [{
                        view: color.createView(), loadOp: "clear", clearValue: [0, 0, 0, 1], storeOp: "store",
                    }],
                    depthStencilAttachment: {
                        view: depthStencil.createView(),
                        depthLoadOp: "clear", depthClearValue: 1, depthStoreOp: "store",
                        stencilLoadOp: "clear", stencilClearValue: 0, stencilStoreOp: "store",
                    },
                });
                pass.setVertexBuffer(0, vertices);
                pass.setStencilReference(7);
                pass.setPipeline(writePipeline);
                pass.setBindGroup(0, greenGroup);
                pass.draw(3, 1, 0);
                pass.setPipeline(stencilRejectPipeline);
                pass.setBindGroup(0, stencilRejectGroup);
                pass.draw(3, 1, 3);
                pass.setPipeline(depthRejectPipeline);
                pass.setBindGroup(0, depthRejectGroup);
                pass.draw(3, 1, 6);
                pass.end();
                encoder.copyTextureToBuffer({ texture: color }, { buffer: readback, bytesPerRow: 256 }, [4, 4]);
                device.queue.submit([encoder.finish()]);
                return readback.mapAsync(GPUMapMode.READ).then(() => {
                    const pixel = [...new Uint8Array(readback.getMappedRange()).slice(0, 4)];
                    readback.unmap();
                    gpuDepthResult = JSON.stringify({
                        depth: depthStencil instanceof GPUTexture,
                        pipelines: writePipeline instanceof GPURenderPipeline && stencilRejectPipeline instanceof GPURenderPipeline,
                        explicitLayout: pipelineLayout instanceof GPUPipelineLayout,
                        native: pass.setStencilReference.toString(),
                        pixel,
                    });
                });
            });
        }).catch(error => gpuDepthResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page.eval("gpuDepthResult").unwrap().to_string().unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"depth":true,"pipelines":true,"explicitLayout":true,"native":"function setStencilReference() { [native code] }","pixel":[0,255,0,255]}"#,
        "unexpected WebGPU depth/stencil result: {result}",
    );
}

#[test]
fn webgpu_render_pass_dynamic_state_controls_pixels() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuDynamicStateResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuDynamicStateResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const shader = device.createShaderModule({ code: `
                    @vertex fn vertexMain(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                        var positions = array<vec2f, 3>(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
                        return vec4f(positions[index], 0, 1);
                    }
                    @fragment fn fragmentMain() -> @location(0) vec4f {
                        return vec4f(1, 1, 1, 1);
                    }
                ` });
                const pipeline = device.createRenderPipeline({
                    layout: "auto",
                    vertex: { module: shader, entryPoint: "vertexMain" },
                    fragment: { module: shader, entryPoint: "fragmentMain", targets: [{
                        format: "rgba8unorm",
                        blend: {
                            color: { srcFactor: "constant", dstFactor: "zero", operation: "add" },
                            alpha: { srcFactor: "one", dstFactor: "zero", operation: "add" },
                        },
                    }] },
                });
                const texture = device.createTexture({
                    size: [4, 4], format: "rgba8unorm",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
                });
                const readback = device.createBuffer({
                    size: 256 * 4, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const encoder = device.createCommandEncoder();
                const pass = encoder.beginRenderPass({ colorAttachments: [{
                    view: texture.createView(), loadOp: "clear",
                    clearValue: [0, 0, 0, 1], storeOp: "store",
                }] });
                pass.setPipeline(pipeline);
                pass.setViewport(0, 0, 2, 4, 0, 1);
                pass.setScissorRect(0, 1, 4, 2);
                pass.setBlendConstant({ r: 0, g: 1, b: 0, a: 1 });
                pass.draw(3);
                pass.end();
                encoder.copyTextureToBuffer(
                    { texture }, { buffer: readback, bytesPerRow: 256, rowsPerImage: 4 }, [4, 4],
                );
                device.queue.submit([encoder.finish()]);
                return readback.mapAsync(GPUMapMode.READ).then(() => {
                    const bytes = new Uint8Array(readback.getMappedRange());
                    const pixel = (x, y) => [...bytes.slice(y * 256 + x * 4, y * 256 + x * 4 + 4)];
                    gpuDynamicStateResult = JSON.stringify({
                        inside: pixel(0, 1),
                        outsideViewport: pixel(2, 1),
                        outsideScissor: pixel(0, 0),
                        native: [
                            pass.setViewport.toString(),
                            pass.setScissorRect.toString(),
                            pass.setBlendConstant.toString(),
                        ],
                    });
                    readback.unmap();
                });
            });
        }).catch(error => gpuDynamicStateResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuDynamicStateResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"inside":[0,255,0,255],"outsideViewport":[0,0,0,255],"outsideScissor":[0,0,0,255],"native":["function setViewport() { [native code] }","function setScissorRect() { [native code] }","function setBlendConstant() { [native code] }"]}"#,
        "unexpected WebGPU render dynamic-state result: {result}",
    );
}

#[test]
fn webgpu_dynamic_bind_group_offsets_select_compute_and_render_uniforms() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuDynamicOffsetResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuDynamicOffsetResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const stride = device.limits.minUniformBufferOffsetAlignment;
                const uniformData = new ArrayBuffer(stride + 16);
                new Float32Array(uniformData, 0, 4).set([1, 0, 0, 1]);
                new Float32Array(uniformData, stride, 4).set([0, 1, 0, 1]);
                const uniforms = device.createBuffer({
                    size: uniformData.byteLength,
                    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(uniforms, 0, uniformData);
                const output = device.createBuffer({
                    size: 16, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
                });
                const outputReadback = device.createBuffer({
                    size: 16, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const computeLayout = device.createBindGroupLayout({ entries: [
                    { binding: 0, visibility: GPUShaderStage.COMPUTE, buffer: {
                        type: "uniform", hasDynamicOffset: true, minBindingSize: 16,
                    } },
                    { binding: 1, visibility: GPUShaderStage.COMPUTE, buffer: {
                        type: "storage", minBindingSize: 16,
                    } },
                ] });
                const computePipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [computeLayout] });
                const computeShader = device.createShaderModule({ code: `
                    struct Color { value: vec4f }
                    @group(0) @binding(0) var<uniform> color: Color;
                    @group(0) @binding(1) var<storage, read_write> output: Color;
                    @compute @workgroup_size(1) fn main() { output.value = color.value; }
                ` });
                const computePipeline = device.createComputePipeline({
                    layout: computePipelineLayout,
                    compute: { module: computeShader, entryPoint: "main" },
                });
                const computeGroup = device.createBindGroup({
                    layout: computeLayout,
                    entries: [
                        { binding: 0, resource: { buffer: uniforms, size: 16 } },
                        { binding: 1, resource: { buffer: output, size: 16 } },
                    ],
                });

                const renderLayout = device.createBindGroupLayout({ entries: [{
                    binding: 0, visibility: GPUShaderStage.FRAGMENT, buffer: {
                        type: "uniform", hasDynamicOffset: true, minBindingSize: 16,
                    },
                }] });
                const renderPipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [renderLayout] });
                const renderShader = device.createShaderModule({ code: `
                    struct Color { value: vec4f }
                    @group(0) @binding(0) var<uniform> color: Color;
                    @vertex fn vertexMain(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                        var positions = array<vec2f, 3>(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
                        return vec4f(positions[index], 0, 1);
                    }
                    @fragment fn fragmentMain() -> @location(0) vec4f { return color.value; }
                ` });
                const renderPipeline = device.createRenderPipeline({
                    layout: renderPipelineLayout,
                    vertex: { module: renderShader, entryPoint: "vertexMain" },
                    fragment: { module: renderShader, entryPoint: "fragmentMain", targets: [{ format: "rgba8unorm" }] },
                });
                const renderGroup = device.createBindGroup({
                    layout: renderLayout,
                    entries: [{ binding: 0, resource: { buffer: uniforms, size: 16 } }],
                });
                const texture = device.createTexture({
                    size: [1, 1], format: "rgba8unorm",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
                });
                const textureReadback = device.createBuffer({
                    size: 256, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });

                const encoder = device.createCommandEncoder();
                const computePass = encoder.beginComputePass();
                computePass.setPipeline(computePipeline);
                computePass.setBindGroup(0, computeGroup, new Uint32Array([17, stride, 23]), 1, 1);
                computePass.dispatchWorkgroups(1);
                computePass.end();
                const renderPass = encoder.beginRenderPass({ colorAttachments: [{
                    view: texture.createView(), loadOp: "clear", clearValue: [0, 0, 0, 1], storeOp: "store",
                }] });
                renderPass.setPipeline(renderPipeline);
                renderPass.setBindGroup(0, renderGroup, [stride]);
                renderPass.draw(3);
                renderPass.end();
                encoder.copyBufferToBuffer(output, 0, outputReadback, 0, 16);
                encoder.copyTextureToBuffer(
                    { texture }, { buffer: textureReadback, bytesPerRow: 256 }, [1, 1],
                );
                device.queue.submit([encoder.finish()]);
                return Promise.all([
                    outputReadback.mapAsync(GPUMapMode.READ),
                    textureReadback.mapAsync(GPUMapMode.READ),
                ]).then(() => {
                    gpuDynamicOffsetResult = JSON.stringify({
                        alignment: stride > 0,
                        compute: [...new Float32Array(outputReadback.getMappedRange())],
                        render: [...new Uint8Array(textureReadback.getMappedRange()).slice(0, 4)],
                        native: [computePass.setBindGroup.toString(), renderPass.setBindGroup.toString()],
                    });
                    outputReadback.unmap();
                    textureReadback.unmap();
                });
            });
        }).catch(error => gpuDynamicOffsetResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuDynamicOffsetResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"alignment":true,"compute":[0,1,0,1],"render":[0,255,0,255],"native":["function setBindGroup() { [native code] }","function setBindGroup() { [native code] }"]}"#,
        "unexpected WebGPU dynamic-offset result: {result}",
    );
}

#[test]
fn webgpu_ordered_compute_state_and_indirect_commands_execute() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuIndirectResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuIndirectResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const computeLayout = device.createBindGroupLayout({ entries: [{
                    binding: 0, visibility: GPUShaderStage.COMPUTE,
                    buffer: { type: "storage", minBindingSize: 4 },
                }] });
                const computePipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [computeLayout] });
                const shader = expression => device.createShaderModule({ code: `
                    @group(0) @binding(0) var<storage, read_write> value: u32;
                    @compute @workgroup_size(1) fn main() { value = ${expression}; }
                ` });
                const addPipeline = device.createComputePipeline({
                    layout: computePipelineLayout,
                    compute: { module: shader("value + 1u"), entryPoint: "main" },
                });
                const doublePipeline = device.createComputePipeline({
                    layout: computePipelineLayout,
                    compute: { module: shader("value * 2u"), entryPoint: "main" },
                });
                const output = device.createBuffer({
                    size: 4, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(output, 0, new Uint32Array([0]));
                const group = device.createBindGroup({
                    layout: computeLayout, entries: [{ binding: 0, resource: { buffer: output } }],
                });
                const computeIndirect = device.createBuffer({
                    size: 12, usage: GPUBufferUsage.INDIRECT | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(computeIndirect, 0, new Uint32Array([1, 1, 1]));
                const outputReadback = device.createBuffer({
                    size: 4, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });

                const renderShader = device.createShaderModule({ code: `
                    @vertex fn vertexMain(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                        var positions = array<vec2f, 3>(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
                        return vec4f(positions[index], 0, 1);
                    }
                    @fragment fn fragmentMain() -> @location(0) vec4f { return vec4f(0, 1, 0, 1); }
                ` });
                const renderPipeline = device.createRenderPipeline({
                    layout: "auto",
                    vertex: { module: renderShader, entryPoint: "vertexMain" },
                    fragment: { module: renderShader, entryPoint: "fragmentMain", targets: [{ format: "rgba8unorm" }] },
                });
                const renderIndirect = device.createBuffer({
                    size: 36, usage: GPUBufferUsage.INDIRECT | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(renderIndirect, 0, new Uint32Array([
                    3, 1, 0, 0,
                    3, 1, 0, 0, 0,
                ]));
                const indices = device.createBuffer({
                    size: 8, usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
                });
                device.queue.writeBuffer(indices, 0, new Uint16Array([0, 1, 2, 0]));
                const texture = device.createTexture({
                    size: [2, 1], format: "rgba8unorm",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
                });
                const textureReadback = device.createBuffer({
                    size: 256, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });

                const encoder = device.createCommandEncoder();
                const computePass = encoder.beginComputePass();
                computePass.setPipeline(addPipeline);
                computePass.setBindGroup(0, group);
                computePass.dispatchWorkgroups(1);
                computePass.setPipeline(doublePipeline);
                computePass.setBindGroup(0, group);
                computePass.dispatchWorkgroupsIndirect(computeIndirect, 0);
                computePass.end();

                const renderPass = encoder.beginRenderPass({ colorAttachments: [{
                    view: texture.createView(), loadOp: "clear", clearValue: [0, 0, 0, 1], storeOp: "store",
                }] });
                renderPass.setPipeline(renderPipeline);
                renderPass.setScissorRect(0, 0, 1, 1);
                renderPass.drawIndirect(renderIndirect, 0);
                renderPass.setScissorRect(1, 0, 1, 1);
                renderPass.setIndexBuffer(indices, "uint16");
                renderPass.drawIndexedIndirect(renderIndirect, 16);
                renderPass.end();
                encoder.copyBufferToBuffer(output, 0, outputReadback, 0, 4);
                encoder.copyTextureToBuffer(
                    { texture }, { buffer: textureReadback, bytesPerRow: 256 }, [2, 1],
                );
                device.queue.submit([encoder.finish()]);
                return Promise.all([
                    outputReadback.mapAsync(GPUMapMode.READ),
                    textureReadback.mapAsync(GPUMapMode.READ),
                ]).then(() => {
                    const textureBytes = new Uint8Array(textureReadback.getMappedRange());
                    gpuIndirectResult = JSON.stringify({
                        orderedCompute: new Uint32Array(outputReadback.getMappedRange())[0],
                        pixels: [...textureBytes.slice(0, 8)],
                        native: [
                            computePass.dispatchWorkgroupsIndirect.toString(),
                            renderPass.drawIndirect.toString(),
                            renderPass.drawIndexedIndirect.toString(),
                        ],
                    });
                    outputReadback.unmap();
                    textureReadback.unmap();
                });
            });
        }).catch(error => gpuIndirectResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page.eval("gpuIndirectResult").unwrap().to_string().unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"orderedCompute":2,"pixels":[0,255,0,255,0,255,0,255],"native":["function dispatchWorkgroupsIndirect() { [native code] }","function drawIndirect() { [native code] }","function drawIndexedIndirect() { [native code] }"]}"#,
        "unexpected WebGPU indirect-command result: {result}",
    );
}

#[test]
fn webgpu_error_scopes_return_native_validation_errors() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuErrorResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuErrorResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                device.pushErrorScope("validation");
                const invalid = device.createBuffer({
                    size: device.limits.maxBufferSize + 4,
                    usage: GPUBufferUsage.COPY_DST,
                });
                return device.popErrorScope().then(error => {
                    device.pushErrorScope("validation");
                    return device.popErrorScope().then(empty => {
                        const manual = new GPUValidationError("manual");
                        gpuErrorResult = JSON.stringify({
                            invalid: invalid instanceof GPUBuffer,
                            captured: error instanceof GPUValidationError,
                            base: error instanceof GPUError,
                            message: error.message.length > 0,
                            empty: empty === null,
                            manual: manual.message,
                            native: device.pushErrorScope.toString(),
                        });
                    });
                });
            });
        }).catch(error => gpuErrorResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page.eval("gpuErrorResult").unwrap().to_string().unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"invalid":true,"captured":true,"base":true,"message":true,"empty":true,"manual":"manual","native":"function pushErrorScope() { [native code] }"}"#,
        "unexpected WebGPU error-scope result: {result}",
    );
}

#[test]
fn webgpu_uncaptured_validation_errors_dispatch_device_events() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuUncapturedResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuUncapturedResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                device.onuncapturederror = event => {
                    gpuUncapturedResult = JSON.stringify({
                        target: event.target === device,
                        deviceEventTarget: device instanceof EventTarget,
                        event: event instanceof GPUUncapturedErrorEvent,
                        error: event.error instanceof GPUValidationError,
                        message: event.error.message.length > 0,
                        trusted: event.isTrusted,
                        native: device.addEventListener.toString(),
                    });
                };
                const invalid = device.createBuffer({
                    size: device.limits.maxBufferSize + 4,
                    usage: GPUBufferUsage.COPY_DST,
                });
                if (!(invalid instanceof GPUBuffer)) gpuUncapturedResult = "invalid-buffer-shape";
            });
        }).catch(error => gpuUncapturedResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuUncapturedResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"target":true,"deviceEventTarget":true,"event":true,"error":true,"message":true,"trusted":true,"native":"function addEventListener() { [native code] }"}"#,
        "unexpected WebGPU uncaptured-error result: {result}",
    );
}

#[test]
fn webgpu_device_destroy_resolves_stable_lost_promise() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuDeviceLostResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuDeviceLostResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const lost = device.lost;
                const stable = lost === device.lost;
                let illegalConstructor = false;
                try { new GPUDeviceLostInfo(); } catch (error) { illegalConstructor = error instanceof TypeError; }
                device.destroy();
                device.destroy();
                return lost.then(info => {
                    gpuDeviceLostResult = JSON.stringify({
                        stable,
                        promise: lost instanceof Promise,
                        info: info instanceof GPUDeviceLostInfo,
                        reason: info.reason,
                        message: info.message.length > 0,
                        frozen: Object.isFrozen(info),
                        illegalConstructor,
                        native: device.destroy.toString(),
                    });
                });
            });
        }).catch(error => gpuDeviceLostResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuDeviceLostResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"stable":true,"promise":true,"info":true,"reason":"destroyed","message":true,"frozen":true,"illegalConstructor":true,"native":"function destroy() { [native code] }"}"#,
        "unexpected WebGPU device-lost result: {result}",
    );
}

#[test]
fn webgpu_explicit_pipeline_layout_executes_compute_work() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuLayoutResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuLayoutResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const bindGroupLayout = device.createBindGroupLayout({
                    entries: [{
                        binding: 0,
                        visibility: GPUShaderStage.COMPUTE,
                        buffer: { type: "storage", minBindingSize: 16 },
                    }],
                });
                const pipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [bindGroupLayout] });
                const shader = device.createShaderModule({ code: `
                    @group(0) @binding(0) var<storage, read_write> values: array<u32>;
                    @compute @workgroup_size(4)
                    fn main(@builtin(global_invocation_id) id: vec3u) {
                        values[id.x] += 10u;
                    }
                ` });
                const pipeline = device.createComputePipeline({
                    layout: pipelineLayout,
                    compute: { module: shader, entryPoint: "main" },
                });
                const storage = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
                });
                const readback = device.createBuffer({
                    size: 16,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                device.queue.writeBuffer(storage, 0, new Uint32Array([1, 2, 3, 4]));
                const group = device.createBindGroup({
                    layout: bindGroupLayout,
                    entries: [{ binding: 0, resource: { buffer: storage } }],
                });
                const encoder = device.createCommandEncoder();
                const pass = encoder.beginComputePass();
                pass.setPipeline(pipeline);
                pass.setBindGroup(0, group);
                pass.dispatchWorkgroups(1);
                pass.end();
                encoder.copyBufferToBuffer(storage, 0, readback, 0, 16);
                device.queue.submit([encoder.finish()]);
                return readback.mapAsync(GPUMapMode.READ).then(() => {
                    const values = [...new Uint32Array(readback.getMappedRange())];
                    readback.unmap();
                    gpuLayoutResult = JSON.stringify({
                        bindGroupLayout: bindGroupLayout instanceof GPUBindGroupLayout,
                        pipelineLayout: pipelineLayout instanceof GPUPipelineLayout,
                        pipeline: pipeline instanceof GPUComputePipeline,
                        values,
                        native: device.createPipelineLayout.toString(),
                    });
                });
            });
        }).catch(error => gpuLayoutResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page.eval("gpuLayoutResult").unwrap().to_string().unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"bindGroupLayout":true,"pipelineLayout":true,"pipeline":true,"values":[11,12,13,14],"native":"function createPipelineLayout() { [native code] }"}"#,
        "unexpected WebGPU explicit-layout result: {result}",
    );
}

#[test]
fn webgpu_async_pipeline_creation_resolves_and_rejects_without_uncaptured_errors() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuAsyncPipelineResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuAsyncPipelineResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                let uncaptured = 0;
                device.addEventListener("uncapturederror", () => uncaptured++);
                const computeModule = device.createShaderModule({ code:
                    "@compute @workgroup_size(1) fn main() {}"
                });
                const renderModule = device.createShaderModule({ code: `
                    @vertex fn vertexMain(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                        var positions = array<vec2f, 3>(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
                        return vec4f(positions[index], 0, 1);
                    }
                    @fragment fn fragmentMain() -> @location(0) vec4f {
                        return vec4f(0, 1, 0, 1);
                    }
                ` });
                const computePromise = device.createComputePipelineAsync({
                    layout: "auto", compute: { module: computeModule, entryPoint: "main" },
                });
                const renderPromise = device.createRenderPipelineAsync({
                    layout: "auto",
                    vertex: { module: renderModule, entryPoint: "vertexMain" },
                    fragment: { module: renderModule, entryPoint: "fragmentMain", targets: [{ format: "rgba8unorm" }] },
                });
                return Promise.all([computePromise, renderPromise]).then(([compute, render]) => {
                    const rejected = device.createComputePipelineAsync({
                        layout: "auto", compute: { module: computeModule, entryPoint: "missing" },
                    });
                    return rejected.then(
                        () => { throw new Error("invalid async pipeline unexpectedly resolved"); },
                        error => device.queue.onSubmittedWorkDone().then(() => {
                            gpuAsyncPipelineResult = JSON.stringify({
                                promised: computePromise instanceof Promise && renderPromise instanceof Promise,
                                compute: compute instanceof GPUComputePipeline,
                                render: render instanceof GPURenderPipeline,
                                error: error instanceof GPUPipelineError,
                                dom: error instanceof DOMException,
                                reason: error.reason,
                                name: error.name,
                                message: error.message.length > 0,
                                uncaptured,
                                native: [
                                    device.createComputePipelineAsync.toString(),
                                    device.createRenderPipelineAsync.toString(),
                                ],
                            });
                        }),
                    );
                });
            });
        }).catch(error => gpuAsyncPipelineResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuAsyncPipelineResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"promised":true,"compute":true,"render":true,"error":true,"dom":true,"reason":"validation","name":"GPUPipelineError","message":true,"uncaptured":0,"native":["function createComputePipelineAsync() { [native code] }","function createRenderPipelineAsync() { [native code] }"]}"#,
        "unexpected WebGPU async-pipeline result: {result}",
    );
}

#[test]
fn webgpu_multisample_render_pass_resolves_color() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuMultisampleResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuMultisampleResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const shader = device.createShaderModule({ code: `
                    @vertex fn vertexMain(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                        var positions = array<vec2f, 3>(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
                        return vec4f(positions[index], 0, 1);
                    }
                    @fragment fn fragmentMain() -> @location(0) vec4f { return vec4f(0, 1, 0, 1); }
                ` });
                const pipeline = device.createRenderPipeline({
                    layout: "auto",
                    vertex: { module: shader, entryPoint: "vertexMain" },
                    fragment: { module: shader, entryPoint: "fragmentMain", targets: [{ format: "rgba8unorm" }] },
                    multisample: { count: 4, mask: 0xffffffff, alphaToCoverageEnabled: false },
                });
                const multisampled = device.createTexture({
                    size: [4, 4], sampleCount: 4, format: "rgba8unorm",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT,
                });
                const resolved = device.createTexture({
                    size: [4, 4], format: "rgba8unorm",
                    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
                });
                const readback = device.createBuffer({
                    size: 256 * 4,
                    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const encoder = device.createCommandEncoder();
                const pass = encoder.beginRenderPass({
                    colorAttachments: [{
                        view: multisampled.createView(),
                        resolveTarget: resolved.createView(),
                        loadOp: "clear", clearValue: [1, 0, 0, 1], storeOp: "discard",
                    }],
                });
                pass.setPipeline(pipeline);
                pass.draw(3);
                pass.end();
                encoder.copyTextureToBuffer({ texture: resolved }, { buffer: readback, bytesPerRow: 256 }, [4, 4]);
                device.queue.submit([encoder.finish()]);
                return readback.mapAsync(GPUMapMode.READ).then(() => {
                    const pixel = [...new Uint8Array(readback.getMappedRange()).slice(0, 4)];
                    readback.unmap();
                    gpuMultisampleResult = JSON.stringify({
                        samples: multisampled.sampleCount,
                        pipeline: pipeline instanceof GPURenderPipeline,
                        pixel,
                    });
                });
            });
        }).catch(error => gpuMultisampleResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuMultisampleResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result == r#"{"samples":4,"pipeline":true,"pixel":[0,255,0,255]}"#,
        "unexpected WebGPU multisample result: {result}",
    );
}

#[test]
fn webgpu_occlusion_query_resolves_visibility() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuQueryResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuQueryResult = "no-adapter"; return; }
            return adapter.requestDevice().then(device => {
                const querySet = device.createQuerySet({ type: "occlusion", count: 1 });
                const shader = device.createShaderModule({ code: `
                    @vertex fn vertexMain(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                        var positions = array<vec2f, 3>(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
                        return vec4f(positions[index], 0, 1);
                    }
                    @fragment fn fragmentMain() -> @location(0) vec4f { return vec4f(0, 1, 0, 1); }
                ` });
                const pipeline = device.createRenderPipeline({
                    layout: "auto",
                    vertex: { module: shader, entryPoint: "vertexMain" },
                    fragment: { module: shader, entryPoint: "fragmentMain", targets: [{ format: "rgba8unorm" }] },
                });
                const color = device.createTexture({
                    size: [4, 4], format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT,
                });
                const queryBuffer = device.createBuffer({
                    size: 8, usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC,
                });
                const readback = device.createBuffer({
                    size: 8, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const encoder = device.createCommandEncoder();
                const pass = encoder.beginRenderPass({
                    colorAttachments: [{ view: color.createView(), loadOp: "clear", clearValue: [0, 0, 0, 1], storeOp: "discard" }],
                    occlusionQuerySet: querySet,
                });
                pass.setPipeline(pipeline);
                pass.beginOcclusionQuery(0);
                pass.draw(3);
                pass.endOcclusionQuery();
                pass.end();
                encoder.resolveQuerySet(querySet, 0, 1, queryBuffer, 0);
                encoder.copyBufferToBuffer(queryBuffer, 0, readback, 0, 8);
                device.queue.submit([encoder.finish()]);
                return readback.mapAsync(GPUMapMode.READ).then(() => {
                    const words = new Uint32Array(readback.getMappedRange());
                    const visible = words[0] !== 0 || words[1] !== 0;
                    readback.unmap();
                    querySet.destroy();
                    gpuQueryResult = JSON.stringify({
                        query: querySet instanceof GPUQuerySet,
                        type: querySet.type,
                        count: querySet.count,
                        visible,
                        native: encoder.resolveQuerySet.toString(),
                    });
                });
            });
        }).catch(error => gpuQueryResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page.eval("gpuQueryResult").unwrap().to_string().unwrap();
    assert!(
        result == "no-adapter"
            || result
                == r#"{"query":true,"type":"occlusion","count":1,"visible":true,"native":"function resolveQuerySet() { [native code] }"}"#,
        "unexpected WebGPU query result: {result}",
    );
}

#[test]
fn webgpu_timestamp_query_is_feature_gated_and_resolves() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgpu(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.gpuTimestampResult = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
            if (!adapter) { gpuTimestampResult = "no-adapter"; return; }
            if (!adapter.features.has("timestamp-query")) { gpuTimestampResult = "no-feature"; return; }
            return adapter.requestDevice({ requiredFeatures: ["timestamp-query"] }).then(device => {
                const querySet = device.createQuerySet({ type: "timestamp", count: 2 });
                const module = device.createShaderModule({ code: "@compute @workgroup_size(1) fn main() {}" });
                const pipeline = device.createComputePipeline({
                    layout: "auto", compute: { module, entryPoint: "main" },
                });
                const queryBuffer = device.createBuffer({
                    size: 16, usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC,
                });
                const readback = device.createBuffer({
                    size: 16, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
                });
                const encoder = device.createCommandEncoder();
                const pass = encoder.beginComputePass({ timestampWrites: {
                    querySet, beginningOfPassWriteIndex: 0, endOfPassWriteIndex: 1,
                }});
                pass.setPipeline(pipeline);
                pass.dispatchWorkgroups(1);
                pass.end();
                encoder.resolveQuerySet(querySet, 0, 2, queryBuffer, 0);
                encoder.copyBufferToBuffer(queryBuffer, 0, readback, 0, 16);
                device.queue.submit([encoder.finish()]);
                return readback.mapAsync(GPUMapMode.READ).then(() => {
                    const words = new Uint32Array(readback.getMappedRange());
                    const ordered = words[3] > words[1] || (words[3] === words[1] && words[2] >= words[0]);
                    const resolved = words.length === 4;
                    readback.unmap();
                    gpuTimestampResult = JSON.stringify({
                        adapterFeature: adapter.features.has("timestamp-query"),
                        deviceFeature: device.features.has("timestamp-query"),
                        type: querySet.type,
                        count: querySet.count,
                        resolved,
                        ordered,
                    });
                });
            });
        }).catch(error => gpuTimestampResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("gpuTimestampResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert!(
        result == "no-adapter"
            || result == "no-feature"
            || result
                == r#"{"adapterFeature":true,"deviceFeature":true,"type":"timestamp","count":2,"resolved":true,"ordered":true}"#,
        "unexpected WebGPU timestamp result: {result}",
    );
}
