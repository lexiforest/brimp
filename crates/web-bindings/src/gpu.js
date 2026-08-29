(() => {
"use strict";

const host = globalThis.__brimpGpuHost;
let graphicsPersona = null;
const devicesById = new Map();
const call = (operation, ...arguments_) => {
    try {
        return host(operation, globalThis, ...arguments_);
    } finally {
        if (operation !== "gpuRequestDevice" && operation !== "gpuTakeUncapturedErrors" &&
            operation !== "gpuTakeDeviceLost") {
            const device = devicesById.get(arguments_[0]);
            device?.__scheduleUncapturedErrorDrain();
            device?.__scheduleDeviceLostDrain();
        }
    }
};
const construct = Symbol("WebGPU construction");
const canvasContexts = new Set();
const minimumGpuLimits = new Set([
    "minUniformBufferOffsetAlignment",
    "minStorageBufferOffsetAlignment",
]);
const extent3D = value => Array.isArray(value) || ArrayBuffer.isView(value)
    ? [Number(value[0]) >>> 0, Number(value[1] ?? 1) >>> 0, Number(value[2] ?? 1) >>> 0]
    : [Number(value?.width) >>> 0, Number(value?.height ?? 1) >>> 0, Number(value?.depthOrArrayLayers ?? 1) >>> 0];
const origin3D = value => Array.isArray(value) || ArrayBuffer.isView(value)
    ? [Number(value[0] ?? 0) >>> 0, Number(value[1] ?? 0) >>> 0, Number(value[2] ?? 0) >>> 0]
    : [Number(value?.x ?? 0) >>> 0, Number(value?.y ?? 0) >>> 0, Number(value?.z ?? 0) >>> 0];
const origin2D = value => Array.isArray(value) || ArrayBuffer.isView(value)
    ? [Number(value[0] ?? 0) >>> 0, Number(value[1] ?? 0) >>> 0]
    : [Number(value?.x ?? 0) >>> 0, Number(value?.y ?? 0) >>> 0];
const externalImageSource = source => {
    let result;
    if (typeof globalThis.ImageData === "function" && source instanceof globalThis.ImageData) {
        result = {
            kind: "image-data", width: source.width, height: source.height, payload: source.data,
            originClean: true, colorSpace: source.colorSpace, pixelFormat: source.pixelFormat,
        };
    } else if (source instanceof HTMLCanvasElement) {
        result = {
            kind: "canvas", width: source.width, height: source.height, payload: source,
            originClean: host("canvasOriginClean", source), colorSpace: "native", pixelFormat: "native",
        };
    } else if (source instanceof HTMLImageElement) {
        const metadata = JSON.parse(host("imageMetadata", source));
        if (!metadata.complete || metadata.width === 0 || metadata.height === 0) {
            throw new DOMException("The image has no decoded image data", "InvalidStateError");
        }
        result = {
            kind: "image", width: metadata.width, height: metadata.height, payload: source,
            originClean: metadata.originClean, colorSpace: "srgb", pixelFormat: "rgba-unorm8",
        };
    } else if (typeof globalThis.ImageBitmap === "function" && source instanceof globalThis.ImageBitmap) {
        if (source.__id === 0) throw new DOMException("The ImageBitmap is closed", "InvalidStateError");
        result = {
            kind: "image-bitmap", width: source.width, height: source.height, payload: source.__id,
            originClean: source.__originClean, colorSpace: "native", pixelFormat: "native",
        };
    } else {
        throw new TypeError("Unsupported GPU external image source");
    }
    if (result.width === 0 || result.height === 0) {
        throw new DOMException("The external image source has no pixels", "InvalidStateError");
    }
    if (!result.originClean) throw new DOMException("The external image source is not origin-clean", "SecurityError");
    return result;
};
const normalizeDynamicOffsets = (value, start = 0, length = undefined) => {
    const offsets = Array.from(value ?? [], offset => Number(offset) >>> 0);
    if (!ArrayBuffer.isView(value)) return offsets;
    start = Number(start) >>> 0;
    length = length === undefined ? offsets.length - start : Number(length) >>> 0;
    if (start > offsets.length || length > offsets.length - start) {
        throw new RangeError("Dynamic offset range is outside the provided array");
    }
    return offsets.slice(start, start + length);
};
const normalizeImmediateData = (rangeOffset, data, dataOffset = 0, dataSize = undefined) => {
    const isView = ArrayBuffer.isView(data);
    const isBuffer = data instanceof ArrayBuffer
        || (typeof SharedArrayBuffer === "function" && data instanceof SharedArrayBuffer);
    if (!isView && !isBuffer) throw new TypeError("Immediate data must be an ArrayBuffer or view");
    rangeOffset = Number(rangeOffset);
    dataOffset = Number(dataOffset);
    const elementSize = isView && Number.isSafeInteger(data.BYTES_PER_ELEMENT)
        ? data.BYTES_PER_ELEMENT : 1;
    const elementLength = data.byteLength / elementSize;
    dataSize = dataSize === undefined ? elementLength - dataOffset : Number(dataSize);
    if (!Number.isSafeInteger(rangeOffset) || rangeOffset < 0 || rangeOffset > 0xffffffff
        || !Number.isSafeInteger(dataOffset) || dataOffset < 0
        || !Number.isSafeInteger(dataSize) || dataSize < 0) {
        throw new TypeError("Immediate data offsets and sizes must be non-negative integers");
    }
    if (dataOffset > elementLength || dataSize > elementLength - dataOffset) {
        throw new DOMException("Immediate data range is outside the provided data", "OperationError");
    }
    const buffer = isView ? data.buffer : data;
    const byteOffset = (isView ? data.byteOffset : 0) + dataOffset * elementSize;
    const byteLength = dataSize * elementSize;
    return {
        offset: rangeOffset,
        data: Array.from(new Uint8Array(buffer, byteOffset, byteLength)),
    };
};

class GPUError {
    constructor(token, message) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "message", { value: String(message), enumerable: true });
    }
}

class GPUValidationError extends GPUError {
    constructor(message) { super(construct, message); }
}

class GPUOutOfMemoryError extends GPUError {
    constructor(message) { super(construct, message); }
}

class GPUInternalError extends GPUError {
    constructor(message) { super(construct, message); }
}

const gpuErrorFromRecord = record => {
    const constructor = record[0] === "validation" ? GPUValidationError
        : record[0] === "out-of-memory" ? GPUOutOfMemoryError
        : GPUInternalError;
    return new constructor(record[1]);
};

class GPUUncapturedErrorEvent extends Event {
    constructor(type, init) {
        if (!init || !(init.error instanceof GPUError)) throw new TypeError("A GPUError is required");
        super(type, init);
        Object.defineProperty(this, "error", { value: init.error, enumerable: true });
    }
}

class GPUDeviceLostInfo {
    constructor(token, values) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            reason: { value: String(values.reason), enumerable: true },
            message: { value: String(values.message), enumerable: true },
        });
        Object.freeze(this);
    }
}

class GPUPipelineError extends DOMException {
    constructor(message = "", options) {
        if (!options || (options.reason !== "validation" && options.reason !== "internal")) {
            throw new TypeError("A valid GPUPipelineError reason is required");
        }
        super(String(message), "GPUPipelineError");
        Object.defineProperty(this, "reason", { value: options.reason, enumerable: true });
    }
}

class GPUSupportedLimits {
    constructor(token, values) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.assign(this, values);
        Object.freeze(this);
    }
}

class GPUSupportedFeatures {
    constructor(token, values) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__values", { value: new Set(Array.from(values, String)) });
    }
}

class WGSLLanguageFeatures {
    constructor(token, values) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__values", { value: new Set(Array.from(values, String)) });
    }
}

function installReadonlySetlike(prototype) {
    const values = function values() { return this.__values.values(); };
    Object.defineProperties(prototype, {
        size: { get() { return this.__values.size; }, enumerable: true, configurable: true },
        has: { value(value) { return this.__values.has(String(value)); }, writable: true, enumerable: true, configurable: true },
        entries: { value() { return this.__values.entries(); }, writable: true, enumerable: true, configurable: true },
        keys: { value() { return this.__values.keys(); }, writable: true, enumerable: true, configurable: true },
        values: { value: values, writable: true, enumerable: true, configurable: true },
        forEach: {
            value(callback, thisArgument = undefined) {
                if (typeof callback !== "function") throw new TypeError("callback must be a function");
                this.__values.forEach(value => callback.call(thisArgument, value, value, this));
            },
            writable: true,
            enumerable: true,
            configurable: true,
        },
        [Symbol.iterator]: { value: values, writable: true, configurable: true },
    });
}
installReadonlySetlike(GPUSupportedFeatures.prototype);
installReadonlySetlike(WGSLLanguageFeatures.prototype);

class GPUAdapterInfo {
    constructor(token, values) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            vendor: { value: String(values.vendor ?? ""), enumerable: true },
            architecture: { value: String(values.architecture ?? ""), enumerable: true },
            device: { value: String(values.device ?? ""), enumerable: true },
            description: { value: String(values.description ?? ""), enumerable: true },
        });
        Object.freeze(this);
    }
}

function applyAdapterPersona(metadata) {
    if (graphicsPersona === null) return metadata;
    metadata.info = {
        vendor: graphicsPersona.webgpu_adapter_vendor,
        architecture: graphicsPersona.webgpu_adapter_architecture,
        device: graphicsPersona.webgpu_adapter_device,
        description: graphicsPersona.webgpu_adapter_description,
    };
    const configuredLimit = Number(graphicsPersona.webgpu_max_bind_groups_plus_vertex_buffers);
    if (Number.isFinite(configuredLimit) && configuredLimit > 0) {
        metadata.limits.maxBindGroupsPlusVertexBuffers = Math.min(
            metadata.limits.maxBindGroupsPlusVertexBuffers,
            configuredLimit,
        );
    }
    return metadata;
}

class GPUCompilationMessage {
    constructor(token, values) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            message: { value: String(values.message), enumerable: true },
            type: { value: String(values.type), enumerable: true },
            lineNum: { value: Number(values.lineNum), enumerable: true },
            linePos: { value: Number(values.linePos), enumerable: true },
            offset: { value: Number(values.offset), enumerable: true },
            length: { value: Number(values.length), enumerable: true },
        });
        Object.freeze(this);
    }
}

class GPUCompilationInfo {
    constructor(token, messages) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "messages", {
            value: Object.freeze(messages.map(message => new GPUCompilationMessage(construct, message))),
            enumerable: true,
        });
        Object.freeze(this);
    }
}

class GPUBuffer {
    constructor(token, device, id, descriptor) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id },
            size: { value: Number(descriptor.size), enumerable: true },
            usage: { value: Number(descriptor.usage) >>> 0, enumerable: true },
            label: { value: String(descriptor.label ?? ""), writable: true, enumerable: true },
            __destroyed: { value: false, writable: true },
            __mapState: { value: descriptor.mappedAtCreation ? "mapped" : "unmapped", writable: true },
            __mappedData: { value: descriptor.mappedAtCreation ? new ArrayBuffer(Number(descriptor.size)) : null, writable: true },
            __mapMode: { value: descriptor.mappedAtCreation ? GPUMapMode.WRITE : 0, writable: true },
            __mapOffset: { value: 0, writable: true },
            __mapSize: { value: descriptor.mappedAtCreation ? Number(descriptor.size) : 0, writable: true },
            __mappedRanges: { value: [], writable: true },
            __mapGeneration: { value: 0, writable: true },
        });
    }
    get mapState() { return this.__mapState; }
    destroy() {
        if (this.__destroyed) return;
        if (this.mapState === "mapped") this.unmap();
        call("gpuDestroyBuffer", this.__device.__id, this.__id);
        this.__destroyed = true;
        this.__mapState = "unmapped";
    }
    mapAsync(mode, offset = 0, size = undefined) {
        if (this.__destroyed) return Promise.reject(new DOMException("GPU buffer is destroyed", "OperationError"));
        if (this.mapState !== "unmapped") return Promise.reject(new DOMException("GPU buffer is already mapped", "OperationError"));
        mode = Number(mode) >>> 0;
        offset = Number(offset);
        size = size === undefined ? this.size - offset : Number(size);
        const validMode = mode === GPUMapMode.READ || mode === GPUMapMode.WRITE;
        const requiredUsage = mode === GPUMapMode.READ ? GPUBufferUsage.MAP_READ : GPUBufferUsage.MAP_WRITE;
        if (!validMode || (this.usage & requiredUsage) === 0 || !Number.isSafeInteger(offset)
            || !Number.isSafeInteger(size) || offset < 0 || size <= 0 || offset % 8 !== 0
            || size % 4 !== 0 || offset + size > this.size) {
            return Promise.reject(new DOMException("Invalid GPU buffer mapping", "OperationError"));
        }
        this.__mapState = "pending";
        const generation = ++this.__mapGeneration;
        return Promise.resolve().then(() => {
            if (generation !== this.__mapGeneration || this.__mapState !== "pending") {
                throw new DOMException("GPU buffer mapping was cancelled", "AbortError");
            }
            const nativeMode = mode === GPUMapMode.READ ? "read" : "write";
            const bytes = call("gpuMapBuffer", this.__device.__id, this.__id, nativeMode, offset, size);
            const copy = new Uint8Array(bytes.byteLength);
            copy.set(bytes);
            this.__mappedData = copy.buffer;
            this.__mapMode = mode;
            this.__mapOffset = offset;
            this.__mapSize = size;
            this.__mappedRanges = [];
            this.__mapState = "mapped";
        }).catch(error => {
            if (generation !== this.__mapGeneration) throw error;
            this.__mapState = "unmapped";
            throw error;
        });
    }
    getMappedRange(offset = 0, size = undefined) {
        if (this.mapState !== "mapped") throw new DOMException("GPU buffer is not mapped", "InvalidStateError");
        offset = Number(offset);
        size = size === undefined ? this.size - offset : Number(size);
        const mapEnd = this.__mapOffset + this.__mapSize;
        if (!Number.isSafeInteger(offset) || !Number.isSafeInteger(size) || offset < 0
            || size <= 0 || offset % 8 !== 0 || size % 4 !== 0
            || offset < this.__mapOffset || offset + size > mapEnd) {
            throw new DOMException("Mapped range is outside the active mapping", "OperationError");
        }
        if (this.__mappedRanges.some(range => offset < range.offset + range.size
            && range.offset < offset + size)) {
            throw new DOMException("Mapped ranges must not overlap", "OperationError");
        }
        const relativeOffset = offset - this.__mapOffset;
        const buffer = relativeOffset === 0 && size === this.__mapSize
            ? this.__mappedData
            : this.__mappedData.slice(relativeOffset, relativeOffset + size);
        this.__mappedRanges.push({ offset, size, buffer });
        return buffer;
    }
    unmap() {
        if (this.mapState === "pending") {
            ++this.__mapGeneration;
            this.__mapState = "unmapped";
            return;
        }
        if (this.mapState !== "mapped") return;
        if (this.__mapMode === GPUMapMode.WRITE) {
            const mapped = new Uint8Array(this.__mappedData);
            for (const range of this.__mappedRanges) {
                mapped.set(new Uint8Array(range.buffer), range.offset - this.__mapOffset);
            }
        }
        const nativeBytes = new Uint8Array(this.__mappedData).slice();
        call(
            "gpuUnmapBuffer", this.__device.__id, this.__id,
            this.__mapMode === GPUMapMode.READ ? "read" : "write",
            this.__mapOffset, nativeBytes,
        );
        for (const buffer of new Set(this.__mappedRanges.map(range => range.buffer))) {
            if (buffer.byteLength > 0 && typeof buffer.transfer === "function") buffer.transfer(0);
        }
        this.__mappedData = null;
        this.__mapMode = 0;
        this.__mapOffset = 0;
        this.__mapSize = 0;
        this.__mappedRanges = [];
        this.__mapState = "unmapped";
    }
}

class GPUTexture {
    constructor(token, device, id, descriptor, size) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id, writable: true },
            width: { value: size[0], enumerable: true },
            height: { value: size[1], enumerable: true },
            depthOrArrayLayers: { value: size[2], enumerable: true },
            mipLevelCount: { value: Number(descriptor.mipLevelCount ?? 1) >>> 0, enumerable: true },
            sampleCount: { value: Number(descriptor.sampleCount ?? 1) >>> 0, enumerable: true },
            dimension: { value: String(descriptor.dimension ?? "2d"), enumerable: true },
            format: { value: String(descriptor.format), enumerable: true },
            usage: { value: Number(descriptor.usage) >>> 0, enumerable: true },
            label: { value: String(descriptor.label ?? ""), writable: true, enumerable: true },
        });
    }
    destroy() {
        if (this.__id === 0) return;
        call("gpuDestroyTexture", this.__device.__id, this.__id);
        this.__id = 0;
    }
    createView(descriptor = {}) {
        if (this.__id === 0) throw new DOMException("GPUTexture is destroyed", "InvalidStateError");
        descriptor ??= {};
        const normalized = {
            label: String(descriptor.label ?? ""),
            format: descriptor.format === undefined ? null : String(descriptor.format),
            dimension: descriptor.dimension === undefined ? null : String(descriptor.dimension),
            usage: descriptor.usage === undefined ? null : Number(descriptor.usage) >>> 0,
            aspect: String(descriptor.aspect ?? "all"),
            base_mip_level: Number(descriptor.baseMipLevel ?? 0) >>> 0,
            mip_level_count: descriptor.mipLevelCount === undefined
                ? null
                : Number(descriptor.mipLevelCount) >>> 0,
            base_array_layer: Number(descriptor.baseArrayLayer ?? 0) >>> 0,
            array_layer_count: descriptor.arrayLayerCount === undefined
                ? null
                : Number(descriptor.arrayLayerCount) >>> 0,
        };
        return new GPUTextureView(
            construct,
            this.__device,
            call("gpuCreateTextureView", this.__device.__id, this.__id, JSON.stringify(normalized)),
            normalized.label,
        );
    }
}

class GPUTextureView {
    constructor(token, device, id, label) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id },
            label: { value: String(label ?? ""), writable: true, enumerable: true },
        });
    }
}

class GPUSampler {
    constructor(token, device, id, label) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id },
            label: { value: String(label ?? ""), writable: true, enumerable: true },
        });
    }
}

class GPUQuerySet {
    constructor(token, device, id, descriptor) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id, writable: true },
            type: { value: String(descriptor.type), enumerable: true },
            count: { value: Number(descriptor.count) >>> 0, enumerable: true },
            label: { value: String(descriptor.label ?? ""), writable: true, enumerable: true },
        });
    }
    destroy() {
        if (this.__id === 0) return;
        call("gpuDestroyQuerySet", this.__device.__id, this.__id);
        this.__id = 0;
    }
}

class GPUCommandBuffer {
    constructor(token, device, id, label) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id, writable: true },
            label: { value: String(label ?? ""), writable: true, enumerable: true },
        });
    }
}

class GPURenderBundle {
    constructor(token, device, id, label) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device },
            __id: { value: id },
            label: { value: String(label ?? ""), writable: true, enumerable: true },
        });
    }
}

class GPUShaderModule {
    constructor(token, device, id, label) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id },
            label: { value: String(label ?? ""), writable: true, enumerable: true },
        });
    }
    getCompilationInfo() {
        try {
            const messages = JSON.parse(call("gpuShaderCompilationInfo", this.__device.__id, this.__id));
            return Promise.resolve(new GPUCompilationInfo(construct, messages));
        } catch (error) {
            return Promise.reject(error);
        }
    }
}

class GPUBindGroupLayout {
    constructor(token, device, id, label = "") {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id },
            label: { value: String(label), writable: true, enumerable: true },
        });
    }
}

class GPUPipelineLayout {
    constructor(token, device, id, label = "") {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id },
            label: { value: String(label), writable: true, enumerable: true },
        });
    }
}

class GPUBindGroup {
    constructor(token, device, id, label) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id },
            label: { value: String(label ?? ""), writable: true, enumerable: true },
        });
    }
}

class GPUComputePipeline {
    constructor(token, device, id, label) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id },
            label: { value: String(label ?? ""), writable: true, enumerable: true },
        });
    }
    getBindGroupLayout(index) {
        return new GPUBindGroupLayout(
            construct,
            this.__device,
            call("gpuComputeBindGroupLayout", this.__device.__id, this.__id, Number(index) >>> 0),
        );
    }
}

class GPURenderPipeline {
    constructor(token, device, id, label) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id },
            label: { value: String(label ?? ""), writable: true, enumerable: true },
        });
    }
    getBindGroupLayout(index) {
        return new GPUBindGroupLayout(
            construct,
            this.__device,
            call("gpuRenderBindGroupLayout", this.__device.__id, this.__id, Number(index) >>> 0),
        );
    }
}

function normalizeTimestampWrites(device, value) {
    if (value === undefined) return null;
    if (!value || !(value.querySet instanceof GPUQuerySet)
        || value.querySet.__device !== device || value.querySet.__id === 0
        || value.querySet.type !== "timestamp") {
        throw new TypeError("Invalid timestamp GPUQuerySet");
    }
    const beginning = value.beginningOfPassWriteIndex === undefined
        ? null : Number(value.beginningOfPassWriteIndex) >>> 0;
    const end = value.endOfPassWriteIndex === undefined
        ? null : Number(value.endOfPassWriteIndex) >>> 0;
    if (beginning === null && end === null) throw new TypeError("At least one timestamp write index is required");
    if ((beginning !== null && beginning >= value.querySet.count)
        || (end !== null && end >= value.querySet.count)) {
        throw new RangeError("Timestamp write index is outside the query set");
    }
    return {
        query_set: value.querySet.__id,
        beginning_of_pass_write_index: beginning,
        end_of_pass_write_index: end,
    };
}

class GPUComputePassEncoder {
    constructor(token, encoder, descriptor) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __encoder: { value: encoder },
            __pipeline: { value: null, writable: true },
            __commands: { value: [] },
            __timestampWrites: { value: normalizeTimestampWrites(encoder.__device, descriptor.timestampWrites) },
            __ended: { value: false, writable: true },
            label: { value: String(descriptor.label ?? ""), writable: true, enumerable: true },
        });
    }
    insertDebugMarker(markerLabel) {
        if (this.__ended) throw new DOMException("GPUComputePassEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "insertDebugMarker", marker_label: String(markerLabel) });
    }
    pushDebugGroup(groupLabel) {
        if (this.__ended) throw new DOMException("GPUComputePassEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "pushDebugGroup", group_label: String(groupLabel) });
    }
    popDebugGroup() {
        if (this.__ended) throw new DOMException("GPUComputePassEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "popDebugGroup" });
    }
    setImmediates(rangeOffset, data, dataOffset = 0, dataSize = undefined) {
        if (this.__ended) throw new DOMException("GPUComputePassEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "setImmediates", ...normalizeImmediateData(rangeOffset, data, dataOffset, dataSize) });
    }
    setPipeline(pipeline) {
        if (this.__ended) throw new DOMException("GPUComputePassEncoder has ended", "InvalidStateError");
        if (!(pipeline instanceof GPUComputePipeline) || pipeline.__device !== this.__encoder.__device) {
            throw new TypeError("Invalid GPUComputePipeline");
        }
        this.__pipeline = pipeline;
        this.__commands.push({ operation: "setPipeline", pipeline: pipeline.__id });
    }
    setBindGroup(index, bindGroup, dynamicOffsets = [], dynamicOffsetsDataStart = 0, dynamicOffsetsDataLength = undefined) {
        if (this.__ended) throw new DOMException("GPUComputePassEncoder has ended", "InvalidStateError");
        if (!(bindGroup instanceof GPUBindGroup) || bindGroup.__device !== this.__encoder.__device) {
            throw new TypeError("Invalid GPUBindGroup");
        }
        this.__commands.push({
            operation: "setBindGroup",
            index: Number(index) >>> 0,
            group: bindGroup.__id,
            dynamic_offsets: normalizeDynamicOffsets(dynamicOffsets, dynamicOffsetsDataStart, dynamicOffsetsDataLength),
        });
    }
    dispatchWorkgroups(x, y = 1, z = 1) {
        if (this.__ended) throw new DOMException("GPUComputePassEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A compute pipeline must be set before dispatch", "InvalidStateError");
        this.__commands.push({
            operation: "dispatchWorkgroups",
            x: Number(x) >>> 0, y: Number(y) >>> 0, z: Number(z) >>> 0,
        });
    }
    dispatchWorkgroupsIndirect(indirectBuffer, indirectOffset) {
        if (this.__ended) throw new DOMException("GPUComputePassEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A compute pipeline must be set before dispatch", "InvalidStateError");
        if (!(indirectBuffer instanceof GPUBuffer) || indirectBuffer.__device !== this.__encoder.__device) {
            throw new TypeError("Invalid GPUBuffer");
        }
        this.__commands.push({
            operation: "dispatchWorkgroupsIndirect",
            buffer: indirectBuffer.__id,
            offset: Number(indirectOffset),
        });
    }
    end() {
        if (this.__ended) throw new DOMException("GPUComputePassEncoder has already ended", "InvalidStateError");
        call(
            "gpuEncodeComputePass",
            this.__encoder.__device.__id,
            this.__encoder.__id,
            JSON.stringify(this.__commands),
            JSON.stringify(this.__timestampWrites),
        );
        this.__ended = true;
        this.__encoder.__activePass = null;
    }
}

class GPURenderBundleEncoder {
    constructor(token, device, descriptor) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        if (!descriptor || typeof descriptor !== "object") {
            throw new TypeError("GPURenderBundleEncoderDescriptor is required");
        }
        const normalized = {
            colorFormats: Array.from(descriptor.colorFormats, format =>
                format === null ? null : String(format)),
            depthStencilFormat: descriptor.depthStencilFormat === undefined
                ? null : String(descriptor.depthStencilFormat),
            depthReadOnly: Boolean(descriptor.depthReadOnly),
            stencilReadOnly: Boolean(descriptor.stencilReadOnly),
            sampleCount: Number(descriptor.sampleCount ?? 1) >>> 0,
        };
        Object.defineProperties(this, {
            __device: { value: device },
            __descriptor: { value: normalized },
            __pipeline: { value: null, writable: true },
            __commands: { value: [] },
            __ended: { value: false, writable: true },
            label: { value: String(descriptor.label ?? ""), writable: true, enumerable: true },
        });
    }
    setImmediates(rangeOffset, data, dataOffset = 0, dataSize = undefined) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "setImmediates", ...normalizeImmediateData(rangeOffset, data, dataOffset, dataSize) });
    }
    setPipeline(pipeline) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has ended", "InvalidStateError");
        if (!(pipeline instanceof GPURenderPipeline) || pipeline.__device !== this.__device) {
            throw new TypeError("Invalid GPURenderPipeline");
        }
        this.__pipeline = pipeline;
        this.__commands.push({ operation: "setPipeline", pipeline: pipeline.__id });
    }
    setBindGroup(index, bindGroup, dynamicOffsets = [], dynamicOffsetsDataStart = 0, dynamicOffsetsDataLength = undefined) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has ended", "InvalidStateError");
        if (!(bindGroup instanceof GPUBindGroup) || bindGroup.__device !== this.__device) {
            throw new TypeError("Invalid GPUBindGroup");
        }
        this.__commands.push({
            operation: "setBindGroup",
            index: Number(index) >>> 0,
            group: bindGroup.__id,
            dynamic_offsets: normalizeDynamicOffsets(dynamicOffsets, dynamicOffsetsDataStart, dynamicOffsetsDataLength),
        });
    }
    setVertexBuffer(slot, buffer, offset = 0, size = undefined) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has ended", "InvalidStateError");
        if (!(buffer instanceof GPUBuffer) || buffer.__device !== this.__device) throw new TypeError("Invalid GPUBuffer");
        this.__commands.push({
            operation: "setVertexBuffer", slot: Number(slot) >>> 0, buffer: buffer.__id,
            offset: Number(offset), size: size === undefined ? null : Number(size),
        });
    }
    setIndexBuffer(buffer, indexFormat, offset = 0, size = undefined) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has ended", "InvalidStateError");
        if (!(buffer instanceof GPUBuffer) || buffer.__device !== this.__device) throw new TypeError("Invalid GPUBuffer");
        this.__commands.push({
            operation: "setIndexBuffer", buffer: buffer.__id, format: String(indexFormat),
            offset: Number(offset), size: size === undefined ? null : Number(size),
        });
    }
    draw(vertexCount, instanceCount = 1, firstVertex = 0, firstInstance = 0) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A render pipeline must be set before drawing", "InvalidStateError");
        this.__commands.push({
            operation: "draw", vertices: Number(vertexCount) >>> 0,
            instances: Number(instanceCount) >>> 0, first_vertex: Number(firstVertex) >>> 0,
            first_instance: Number(firstInstance) >>> 0,
        });
    }
    drawIndexed(indexCount, instanceCount = 1, firstIndex = 0, baseVertex = 0, firstInstance = 0) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A render pipeline must be set before drawing", "InvalidStateError");
        this.__commands.push({
            operation: "drawIndexed", indices: Number(indexCount) >>> 0,
            instances: Number(instanceCount) >>> 0, first_index: Number(firstIndex) >>> 0,
            base_vertex: Number(baseVertex) | 0, first_instance: Number(firstInstance) >>> 0,
        });
    }
    drawIndirect(indirectBuffer, indirectOffset) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A render pipeline must be set before drawing", "InvalidStateError");
        if (!(indirectBuffer instanceof GPUBuffer) || indirectBuffer.__device !== this.__device) throw new TypeError("Invalid GPUBuffer");
        this.__commands.push({
            operation: "drawIndirect", buffer: indirectBuffer.__id, offset: Number(indirectOffset),
        });
    }
    drawIndexedIndirect(indirectBuffer, indirectOffset) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A render pipeline must be set before drawing", "InvalidStateError");
        if (!(indirectBuffer instanceof GPUBuffer) || indirectBuffer.__device !== this.__device) throw new TypeError("Invalid GPUBuffer");
        this.__commands.push({
            operation: "drawIndexedIndirect", buffer: indirectBuffer.__id, offset: Number(indirectOffset),
        });
    }
    finish(descriptor = {}) {
        if (this.__ended) throw new DOMException("GPURenderBundleEncoder has already ended", "InvalidStateError");
        descriptor ??= {};
        const label = String(descriptor.label ?? "");
        const id = call("gpuCreateRenderBundle", this.__device.__id,
            JSON.stringify(this.__descriptor), JSON.stringify(this.__commands), label);
        this.__ended = true;
        return new GPURenderBundle(construct, this.__device, id, label);
    }
}

class GPURenderPassEncoder {
    constructor(token, encoder, descriptor) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        const attachments = Array.from(descriptor.colorAttachments ?? [], attachment => {
            if (!attachment || !(attachment.view instanceof GPUTextureView) || attachment.view.__device !== encoder.__device) {
                throw new TypeError("Invalid GPU render color attachment");
            }
            const clear = attachment.loadOp === "clear";
            if (!clear && attachment.loadOp !== "load") throw new TypeError("GPU attachment loadOp must be 'clear' or 'load'");
            const value = attachment.clearValue ?? {};
            const color = Array.isArray(value) || ArrayBuffer.isView(value)
                ? [Number(value[0] ?? 0), Number(value[1] ?? 0), Number(value[2] ?? 0), Number(value[3] ?? 0)]
                : [Number(value.r ?? 0), Number(value.g ?? 0), Number(value.b ?? 0), Number(value.a ?? 0)];
            const resolveTarget = attachment.resolveTarget ?? null;
            if (resolveTarget !== null && (!(resolveTarget instanceof GPUTextureView) || resolveTarget.__device !== encoder.__device)) {
                throw new TypeError("Invalid GPU resolve target");
            }
            return {
                view: attachment.view.__id,
                resolve_target: resolveTarget?.__id ?? null,
                clear,
                color,
                store: attachment.storeOp !== "discard",
            };
        });
        const depthStencil = descriptor.depthStencilAttachment === undefined ? null : (() => {
            const attachment = descriptor.depthStencilAttachment;
            if (!(attachment?.view instanceof GPUTextureView) || attachment.view.__device !== encoder.__device) throw new TypeError("Invalid GPU depth/stencil attachment");
            const depthReadOnly = Boolean(attachment.depthReadOnly);
            const stencilReadOnly = Boolean(attachment.stencilReadOnly);
            if (!depthReadOnly && !["clear", "load"].includes(attachment.depthLoadOp)) throw new TypeError("Invalid depthLoadOp");
            if (!stencilReadOnly && !["clear", "load"].includes(attachment.stencilLoadOp)) throw new TypeError("Invalid stencilLoadOp");
            return {
                view: attachment.view.__id,
                depth_load: attachment.depthLoadOp === "clear",
                depth_clear_value: Number(attachment.depthClearValue ?? 0),
                depth_store: attachment.depthStoreOp !== "discard",
                depth_read_only: depthReadOnly,
                stencil_load: attachment.stencilLoadOp === "clear",
                stencil_clear_value: Number(attachment.stencilClearValue ?? 0) >>> 0,
                stencil_store: attachment.stencilStoreOp !== "discard",
                stencil_read_only: stencilReadOnly,
            };
        })();
        const occlusionQuerySet = descriptor.occlusionQuerySet ?? null;
        if (occlusionQuerySet !== null && (!(occlusionQuerySet instanceof GPUQuerySet) || occlusionQuerySet.__device !== encoder.__device || occlusionQuerySet.__id === 0)) {
            throw new TypeError("Invalid GPU occlusion query set");
        }
        if (!attachments.length && depthStencil === null) throw new TypeError("At least one render attachment is required");
        const timestampWrites = normalizeTimestampWrites(encoder.__device, descriptor.timestampWrites);
        Object.defineProperties(this, {
            __encoder: { value: encoder },
            __attachments: { value: attachments },
            __depthStencilAttachment: { value: depthStencil },
            __occlusionQuerySet: { value: occlusionQuerySet },
            __timestampWrites: { value: timestampWrites },
            __pipeline: { value: null, writable: true },
            __commands: { value: [] },
            __occlusionQueryActive: { value: false, writable: true },
            __ended: { value: false, writable: true },
            label: { value: String(descriptor.label ?? ""), writable: true, enumerable: true },
        });
    }
    insertDebugMarker(markerLabel) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "insertDebugMarker", marker_label: String(markerLabel) });
    }
    pushDebugGroup(groupLabel) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "pushDebugGroup", group_label: String(groupLabel) });
    }
    popDebugGroup() {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "popDebugGroup" });
    }
    setImmediates(rangeOffset, data, dataOffset = 0, dataSize = undefined) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "setImmediates", ...normalizeImmediateData(rangeOffset, data, dataOffset, dataSize) });
    }
    setPipeline(pipeline) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!(pipeline instanceof GPURenderPipeline) || pipeline.__device !== this.__encoder.__device) {
            throw new TypeError("Invalid GPURenderPipeline");
        }
        this.__pipeline = pipeline;
        this.__commands.push({ operation: "setPipeline", pipeline: pipeline.__id });
    }
    setBindGroup(index, bindGroup, dynamicOffsets = [], dynamicOffsetsDataStart = 0, dynamicOffsetsDataLength = undefined) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!(bindGroup instanceof GPUBindGroup) || bindGroup.__device !== this.__encoder.__device) throw new TypeError("Invalid GPUBindGroup");
        this.__commands.push({
            operation: "setBindGroup",
            index: Number(index) >>> 0,
            group: bindGroup.__id,
            dynamic_offsets: normalizeDynamicOffsets(dynamicOffsets, dynamicOffsetsDataStart, dynamicOffsetsDataLength),
        });
    }
    setVertexBuffer(slot, buffer, offset = 0, size = undefined) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!(buffer instanceof GPUBuffer) || buffer.__device !== this.__encoder.__device) throw new TypeError("Invalid GPUBuffer");
        this.__commands.push({
            operation: "setVertexBuffer",
            slot: Number(slot) >>> 0,
            buffer: buffer.__id,
            offset: Number(offset),
            size: size === undefined ? null : Number(size),
        });
    }
    setIndexBuffer(buffer, indexFormat, offset = 0, size = undefined) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!(buffer instanceof GPUBuffer) || buffer.__device !== this.__encoder.__device) throw new TypeError("Invalid GPUBuffer");
        this.__commands.push({
            operation: "setIndexBuffer",
            buffer: buffer.__id,
            format: String(indexFormat),
            offset: Number(offset),
            size: size === undefined ? null : Number(size),
        });
    }
    draw(vertexCount, instanceCount = 1, firstVertex = 0, firstInstance = 0) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A render pipeline must be set before drawing", "InvalidStateError");
        this.__commands.push({
            operation: "draw",
            vertices: Number(vertexCount) >>> 0,
            instances: Number(instanceCount) >>> 0,
            first_vertex: Number(firstVertex) >>> 0,
            first_instance: Number(firstInstance) >>> 0,
        });
    }
    drawIndexed(indexCount, instanceCount = 1, firstIndex = 0, baseVertex = 0, firstInstance = 0) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A render pipeline must be set before drawing", "InvalidStateError");
        this.__commands.push({
            operation: "drawIndexed",
            indices: Number(indexCount) >>> 0,
            instances: Number(instanceCount) >>> 0,
            first_index: Number(firstIndex) >>> 0,
            base_vertex: Number(baseVertex) | 0,
            first_instance: Number(firstInstance) >>> 0,
        });
    }
    drawIndirect(indirectBuffer, indirectOffset) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A render pipeline must be set before drawing", "InvalidStateError");
        if (!(indirectBuffer instanceof GPUBuffer) || indirectBuffer.__device !== this.__encoder.__device) {
            throw new TypeError("Invalid GPUBuffer");
        }
        this.__commands.push({
            operation: "drawIndirect", buffer: indirectBuffer.__id, offset: Number(indirectOffset),
        });
    }
    drawIndexedIndirect(indirectBuffer, indirectOffset) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!this.__pipeline) throw new DOMException("A render pipeline must be set before drawing", "InvalidStateError");
        if (!(indirectBuffer instanceof GPUBuffer) || indirectBuffer.__device !== this.__encoder.__device) {
            throw new TypeError("Invalid GPUBuffer");
        }
        this.__commands.push({
            operation: "drawIndexedIndirect", buffer: indirectBuffer.__id, offset: Number(indirectOffset),
        });
    }
    executeBundles(bundles) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        const ids = Array.from(bundles, bundle => {
            if (!(bundle instanceof GPURenderBundle) || bundle.__device !== this.__encoder.__device) {
                throw new TypeError("Invalid GPURenderBundle");
            }
            return bundle.__id;
        });
        this.__commands.push({ operation: "executeBundles", bundles: ids });
        this.__pipeline = null;
    }
    setViewport(x, y, width, height, minDepth, maxDepth) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        this.__commands.push({
            operation: "setViewport",
            x: Number(x), y: Number(y), width: Number(width), height: Number(height),
            min_depth: Number(minDepth), max_depth: Number(maxDepth),
        });
    }
    setScissorRect(x, y, width, height) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        this.__commands.push({
            operation: "setScissorRect",
            x: Number(x) >>> 0, y: Number(y) >>> 0,
            width: Number(width) >>> 0, height: Number(height) >>> 0,
        });
    }
    setBlendConstant(color) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        const values = Array.isArray(color) || ArrayBuffer.isView(color)
            ? [color[0], color[1], color[2], color[3]]
            : [color?.r, color?.g, color?.b, color?.a];
        this.__commands.push({ operation: "setBlendConstant", color: values.map(Number) });
    }
    setStencilReference(reference) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        this.__commands.push({ operation: "setStencilReference", reference: Number(reference) >>> 0 });
    }
    beginOcclusionQuery(queryIndex) {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!this.__occlusionQuerySet) throw new DOMException("The render pass has no occlusion query set", "InvalidStateError");
        if (this.__occlusionQueryActive) throw new DOMException("An occlusion query is already active", "InvalidStateError");
        this.__commands.push({ operation: "beginOcclusionQuery", query_index: Number(queryIndex) >>> 0 });
        this.__occlusionQueryActive = true;
    }
    endOcclusionQuery() {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has ended", "InvalidStateError");
        if (!this.__occlusionQueryActive) throw new DOMException("No occlusion query is active", "InvalidStateError");
        this.__commands.push({ operation: "endOcclusionQuery" });
        this.__occlusionQueryActive = false;
    }
    end() {
        if (this.__ended) throw new DOMException("GPURenderPassEncoder has already ended", "InvalidStateError");
        if (this.__occlusionQueryActive) throw new DOMException("An occlusion query is still active", "InvalidStateError");
        call(
            "gpuEncodeRenderPass",
            this.__encoder.__device.__id,
            this.__encoder.__id,
            JSON.stringify(this.__attachments),
            JSON.stringify(this.__depthStencilAttachment),
            JSON.stringify(this.__commands),
            this.__occlusionQuerySet?.__id ?? 0,
            JSON.stringify(this.__timestampWrites),
        );
        this.__ended = true;
        this.__encoder.__activePass = null;
    }
}

class GPUCommandEncoder {
    constructor(token, device, id, label) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __device: { value: device }, __id: { value: id, writable: true },
            __activePass: { value: null, writable: true },
            label: { value: String(label ?? ""), writable: true, enumerable: true },
        });
    }
    insertDebugMarker(markerLabel) {
        if (this.__id === 0 || this.__activePass) throw new DOMException("GPUCommandEncoder is not available", "InvalidStateError");
        call("gpuCommandEncoderInsertDebugMarker", this.__device.__id, this.__id, String(markerLabel));
    }
    pushDebugGroup(groupLabel) {
        if (this.__id === 0 || this.__activePass) throw new DOMException("GPUCommandEncoder is not available", "InvalidStateError");
        call("gpuCommandEncoderPushDebugGroup", this.__device.__id, this.__id, String(groupLabel));
    }
    popDebugGroup() {
        if (this.__id === 0 || this.__activePass) throw new DOMException("GPUCommandEncoder is not available", "InvalidStateError");
        call("gpuCommandEncoderPopDebugGroup", this.__device.__id, this.__id);
    }
    beginComputePass(descriptor = {}) {
        if (this.__id === 0) throw new DOMException("GPUCommandEncoder is already finished", "InvalidStateError");
        if (this.__activePass) throw new DOMException("A pass is already active", "InvalidStateError");
        const pass = new GPUComputePassEncoder(construct, this, descriptor);
        this.__activePass = pass;
        return pass;
    }
    beginRenderPass(descriptor) {
        if (this.__id === 0) throw new DOMException("GPUCommandEncoder is already finished", "InvalidStateError");
        if (this.__activePass) throw new DOMException("A pass is already active", "InvalidStateError");
        const pass = new GPURenderPassEncoder(construct, this, descriptor ?? {});
        this.__activePass = pass;
        return pass;
    }
    copyBufferToBuffer(source, sourceOffset, destination, destinationOffset, size) {
        if (!(source instanceof GPUBuffer) || source.__device !== this.__device ||
            !(destination instanceof GPUBuffer) || destination.__device !== this.__device) {
            throw new TypeError("GPU buffers must belong to this command encoder's device");
        }
        if (this.__id === 0 || this.__activePass) throw new DOMException("GPUCommandEncoder is not available", "InvalidStateError");
        call("gpuCopyBufferToBuffer", this.__device.__id, this.__id, source.__id, Number(sourceOffset), destination.__id, Number(destinationOffset), Number(size));
    }
    clearBuffer(buffer, offset = 0, size = undefined) {
        if (this.__id === 0 || this.__activePass) throw new DOMException("GPUCommandEncoder is not available", "InvalidStateError");
        if (!(buffer instanceof GPUBuffer) || buffer.__device !== this.__device || buffer.__destroyed) {
            throw new TypeError("Invalid GPUBuffer");
        }
        call("gpuClearBuffer", this.__device.__id, this.__id, buffer.__id, Number(offset), size !== undefined,
            size === undefined ? 0 : Number(size));
    }
    copyBufferToTexture(source, destination, copySize) {
        if (this.__id === 0 || this.__activePass) throw new DOMException("GPUCommandEncoder is not available", "InvalidStateError");
        if (!(source?.buffer instanceof GPUBuffer) || source.buffer.__device !== this.__device) throw new TypeError("Invalid source GPUBuffer");
        if (!(destination?.texture instanceof GPUTexture) || destination.texture.__device !== this.__device || destination.texture.__id === 0) {
            throw new TypeError("Invalid destination GPUTexture");
        }
        const origin = origin3D(destination.origin);
        const extent = extent3D(copySize);
        call("gpuCopyBufferToTexture", this.__device.__id, this.__id, source.buffer.__id,
            Number(source.offset ?? 0), Number(source.bytesPerRow ?? 0) >>> 0, Number(source.rowsPerImage ?? 0) >>> 0,
            destination.texture.__id, Number(destination.mipLevel ?? 0) >>> 0,
            ...origin, ...extent);
    }
    copyTextureToBuffer(source, destination, copySize) {
        if (this.__id === 0 || this.__activePass) throw new DOMException("GPUCommandEncoder is not available", "InvalidStateError");
        if (!(source?.texture instanceof GPUTexture) || source.texture.__device !== this.__device || source.texture.__id === 0) {
            throw new TypeError("Invalid source GPUTexture");
        }
        if (!(destination?.buffer instanceof GPUBuffer) || destination.buffer.__device !== this.__device) throw new TypeError("Invalid destination GPUBuffer");
        const origin = origin3D(source.origin);
        const extent = extent3D(copySize);
        call("gpuCopyTextureToBuffer", this.__device.__id, this.__id, source.texture.__id,
            Number(source.mipLevel ?? 0) >>> 0, ...origin, destination.buffer.__id,
            Number(destination.offset ?? 0), Number(destination.bytesPerRow ?? 0) >>> 0, Number(destination.rowsPerImage ?? 0) >>> 0,
            ...extent);
    }
    copyTextureToTexture(source, destination, copySize) {
        if (this.__id === 0 || this.__activePass) throw new DOMException("GPUCommandEncoder is not available", "InvalidStateError");
        if (!(source?.texture instanceof GPUTexture) || source.texture.__device !== this.__device || source.texture.__id === 0) {
            throw new TypeError("Invalid source GPUTexture");
        }
        if (!(destination?.texture instanceof GPUTexture) || destination.texture.__device !== this.__device || destination.texture.__id === 0) {
            throw new TypeError("Invalid destination GPUTexture");
        }
        const sourceOrigin = origin3D(source.origin);
        const destinationOrigin = origin3D(destination.origin);
        const extent = extent3D(copySize);
        call("gpuCopyTextureToTexture", this.__device.__id, this.__id,
            source.texture.__id, Number(source.mipLevel ?? 0) >>> 0, ...sourceOrigin, String(source.aspect ?? "all"),
            destination.texture.__id, Number(destination.mipLevel ?? 0) >>> 0, ...destinationOrigin, String(destination.aspect ?? "all"),
            ...extent);
    }
    resolveQuerySet(querySet, firstQuery, queryCount, destination, destinationOffset) {
        if (this.__id === 0 || this.__activePass) throw new DOMException("GPUCommandEncoder is not available", "InvalidStateError");
        if (!(querySet instanceof GPUQuerySet) || querySet.__device !== this.__device || querySet.__id === 0) throw new TypeError("Invalid GPUQuerySet");
        if (!(destination instanceof GPUBuffer) || destination.__device !== this.__device) throw new TypeError("Invalid destination GPUBuffer");
        call(
            "gpuResolveQuerySet", this.__device.__id, this.__id, querySet.__id,
            Number(firstQuery) >>> 0, Number(queryCount) >>> 0,
            destination.__id, Number(destinationOffset),
        );
    }
    finish(descriptor = {}) {
        if (this.__id === 0) throw new DOMException("GPUCommandEncoder is already finished", "InvalidStateError");
        if (this.__activePass) throw new DOMException("A pass is still active", "InvalidStateError");
        const command = new GPUCommandBuffer(construct, this.__device, call("gpuFinishCommandEncoder", this.__device.__id, this.__id), descriptor.label);
        this.__id = 0;
        return command;
    }
}

class GPUQueue {
    constructor(token, device) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, { __device: { value: device }, label: { value: "", writable: true, enumerable: true } });
    }
    writeBuffer(buffer, bufferOffset, data, dataOffset = 0, size = undefined) {
        if (!(buffer instanceof GPUBuffer) || buffer.__device !== this.__device) throw new TypeError("Invalid GPUBuffer");
        if (!ArrayBuffer.isView(data) && !(data instanceof ArrayBuffer)) throw new TypeError("Data must be an ArrayBuffer or view");
        const view = ArrayBuffer.isView(data) ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength) : new Uint8Array(data);
        dataOffset = Number(dataOffset) >>> 0;
        const bytes = view.subarray(dataOffset, size === undefined ? view.length : dataOffset + (Number(size) >>> 0));
        call("gpuWriteBuffer", this.__device.__id, buffer.__id, Number(bufferOffset), bytes);
    }
    writeTexture(destination, data, dataLayout, size) {
        if (!(destination?.texture instanceof GPUTexture) || destination.texture.__device !== this.__device || destination.texture.__id === 0) throw new TypeError("Invalid GPUTexture");
        if (!ArrayBuffer.isView(data) && !(data instanceof ArrayBuffer)) throw new TypeError("Data must be an ArrayBuffer or view");
        if (!dataLayout || typeof dataLayout !== "object") throw new TypeError("GPUImageDataLayout is required");
        const bytes = ArrayBuffer.isView(data) ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength) : new Uint8Array(data);
        const origin = origin3D(destination.origin ?? [0, 0, 0]);
        const extent = extent3D(size);
        call(
            "gpuWriteTexture", this.__device.__id, destination.texture.__id,
            Number(destination.mipLevel ?? 0) >>> 0, ...origin, String(destination.aspect ?? "all"),
            bytes, Number(dataLayout.offset ?? 0), Number(dataLayout.bytesPerRow ?? 0) >>> 0,
            Number(dataLayout.rowsPerImage ?? 0) >>> 0, ...extent,
        );
    }
    copyExternalImageToTexture(source, destination, copySize) {
        if (!source || typeof source !== "object") throw new TypeError("GPUImageCopyExternalImage is required");
        if (!(destination?.texture instanceof GPUTexture)
            || destination.texture.__device !== this.__device || destination.texture.__id === 0) {
            throw new TypeError("Invalid GPUTexture");
        }
        const external = externalImageSource(source.source);
        const sourceOrigin = origin2D(source.origin ?? [0, 0]);
        const destinationOrigin = origin3D(destination.origin ?? [0, 0, 0]);
        const size = extent3D(copySize);
        if (size[2] !== 1) throw new DOMException("External image copies must have a depth of one", "OperationError");
        const colorSpace = String(destination.colorSpace ?? "srgb");
        if (colorSpace !== "srgb" && colorSpace !== "display-p3") throw new TypeError("Invalid external image colorSpace");
        call(
            "gpuCopyExternalImageToTexture",
            this.__device.__id,
            destination.texture.__id,
            Number(destination.mipLevel ?? 0) >>> 0,
            ...destinationOrigin,
            String(destination.aspect ?? "all"),
            external.kind,
            external.width,
            external.height,
            external.payload,
            Boolean(source.flipY),
            Boolean(destination.premultipliedAlpha),
            external.originClean,
            ...sourceOrigin,
            size[0],
            size[1],
            colorSpace,
            external.colorSpace,
            external.pixelFormat,
        );
    }
    submit(commandBuffers) {
        const commands = Array.from(commandBuffers);
        for (const command of commands) {
            if (!(command instanceof GPUCommandBuffer) || command.__device !== this.__device || command.__id === 0) {
                throw new TypeError("Invalid GPUCommandBuffer");
            }
        }
        call("gpuSubmit", this.__device.__id, JSON.stringify(commands.map(command => command.__id)));
        for (const command of commands) command.__id = 0;
        for (const context of canvasContexts) context.__presentFor(this.__device);
    }
    onSubmittedWorkDone() {
        try {
            call("gpuWaitForSubmittedWork", this.__device.__id);
            return Promise.resolve();
        } catch (error) {
            return Promise.reject(error);
        }
    }
}

class GPUCanvasContext {
    constructor(token, canvas) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            canvas: { value: canvas, enumerable: true },
            __configuration: { value: null, writable: true },
            __currentTexture: { value: null, writable: true },
        });
    }
    configure(configuration) {
        if (!configuration || !(configuration.device instanceof GPUDevice)) throw new TypeError("A GPUDevice is required");
        const format = String(configuration.format);
        if (!["rgba8unorm", "rgba8unorm-srgb", "bgra8unorm", "bgra8unorm-srgb"].includes(format)) {
            throw new DOMException("Unsupported GPU canvas format", "NotSupportedError");
        }
        const alphaMode = String(configuration.alphaMode ?? "opaque");
        if (alphaMode !== "opaque" && alphaMode !== "premultiplied") throw new TypeError("Invalid GPU canvas alphaMode");
        this.__resetState();
        this.__configuration = {
            device: configuration.device,
            format,
            usage: Number(configuration.usage ?? GPUTextureUsage.RENDER_ATTACHMENT) >>> 0,
            alphaMode,
        };
        canvasContexts.add(this);
    }
    unconfigure() {
        this.__resetState();
        this.__configuration = null;
        canvasContexts.delete(this);
    }
    getConfiguration() {
        return this.__configuration ? { ...this.__configuration } : null;
    }
    getCurrentTexture() {
        if (!this.__configuration) throw new DOMException("GPUCanvasContext is not configured", "InvalidStateError");
        if (!this.__currentTexture) {
            this.__currentTexture = this.__configuration.device.createTexture({
                label: "GPUCanvasContext current texture",
                size: [this.canvas.width, this.canvas.height],
                format: this.__configuration.format,
                usage: this.__configuration.usage | GPUTextureUsage.COPY_SRC,
            });
        }
        return this.__currentTexture;
    }
    __presentFor(device) {
        if (!this.__currentTexture || this.__configuration?.device !== device) return;
        const texture = this.__currentTexture;
        call("gpuPresentCanvas", device.__id, texture.__id, this.canvas, this.canvas.width, this.canvas.height);
        texture.destroy();
        this.__currentTexture = null;
    }
    __resetState() {
        this.__currentTexture?.destroy();
        this.__currentTexture = null;
    }
}

class GPUDevice extends EventTarget {
    constructor(token, id, metadata) {
        super();
        if (token !== construct) throw new TypeError("Illegal constructor");
        let resolveLost;
        const lost = new Promise(resolve => { resolveLost = resolve; });
        Object.defineProperties(this, {
            __id: { value: id },
            __uncapturedDrainScheduled: { value: false, writable: true },
            __lostDrainScheduled: { value: false, writable: true },
            __lostSettled: { value: false, writable: true },
            __destroyed: { value: false, writable: true },
            __resolveLost: { value: resolveLost },
            features: { value: new GPUSupportedFeatures(construct, metadata.features ?? []), enumerable: true },
            limits: { value: new GPUSupportedLimits(construct, metadata.limits), enumerable: true },
            queue: { value: new GPUQueue(construct, this), enumerable: true },
            label: { value: String(metadata.label ?? ""), writable: true, enumerable: true },
            lost: { value: lost, enumerable: true },
            onuncapturederror: { value: null, writable: true, enumerable: true },
        });
        devicesById.set(id, this);
    }
    __scheduleUncapturedErrorDrain() {
        if (this.__uncapturedDrainScheduled) return;
        this.__uncapturedDrainScheduled = true;
        queueMicrotask(() => {
            this.__uncapturedDrainScheduled = false;
            const records = JSON.parse(host("gpuTakeUncapturedErrors", globalThis, this.__id));
            for (const record of records) {
                const event = new GPUUncapturedErrorEvent("uncapturederror", { error: gpuErrorFromRecord(record) });
                event.isTrusted = true;
                this.dispatchEvent(event);
            }
        });
    }
    __scheduleDeviceLostDrain() {
        if (this.__lostSettled || this.__lostDrainScheduled) return;
        this.__lostDrainScheduled = true;
        queueMicrotask(() => {
            this.__lostDrainScheduled = false;
            const encoded = host("gpuTakeDeviceLost", globalThis, this.__id);
            if (encoded === null) return;
            this.__lostSettled = true;
            this.__resolveLost(new GPUDeviceLostInfo(construct, JSON.parse(encoded)));
        });
    }
    createBuffer(descriptor) {
        if (!descriptor || typeof descriptor !== "object") throw new TypeError("GPUBufferDescriptor is required");
        const normalized = {
            size: Number(descriptor.size),
            usage: Number(descriptor.usage) >>> 0,
            mappedAtCreation: Boolean(descriptor.mappedAtCreation),
            label: descriptor.label,
        };
        const id = call("gpuCreateBuffer", this.__id, normalized.size, normalized.usage, normalized.mappedAtCreation);
        return new GPUBuffer(construct, this, id, normalized);
    }
    createCommandEncoder(descriptor = {}) {
        return new GPUCommandEncoder(construct, this, call("gpuCreateCommandEncoder", this.__id), descriptor.label);
    }
    createRenderBundleEncoder(descriptor) {
        return new GPURenderBundleEncoder(construct, this, descriptor);
    }
    createTexture(descriptor) {
        if (!descriptor || typeof descriptor !== "object") throw new TypeError("GPUTextureDescriptor is required");
        const size = extent3D(descriptor.size);
        if (size.some(value => value === 0)) throw new RangeError("GPUTexture dimensions must be greater than zero");
        const normalized = {
            ...descriptor,
            mipLevelCount: Number(descriptor.mipLevelCount ?? 1) >>> 0,
            sampleCount: Number(descriptor.sampleCount ?? 1) >>> 0,
            dimension: String(descriptor.dimension ?? "2d"),
            format: String(descriptor.format),
            usage: Number(descriptor.usage) >>> 0,
            viewFormats: Array.from(descriptor.viewFormats ?? [], String),
            label: String(descriptor.label ?? ""),
        };
        const id = call("gpuCreateTexture", this.__id, ...size, normalized.mipLevelCount,
            normalized.sampleCount, normalized.dimension, normalized.format, normalized.usage,
            JSON.stringify(normalized.viewFormats), normalized.label);
        return new GPUTexture(construct, this, id, normalized, size);
    }
    createSampler(descriptor = {}) {
        if (!descriptor || typeof descriptor !== "object") throw new TypeError("GPUSamplerDescriptor must be an object");
        const normalized = {
            addressModeU: String(descriptor.addressModeU ?? "clamp-to-edge"),
            addressModeV: String(descriptor.addressModeV ?? "clamp-to-edge"),
            addressModeW: String(descriptor.addressModeW ?? "clamp-to-edge"),
            magFilter: String(descriptor.magFilter ?? "nearest"),
            minFilter: String(descriptor.minFilter ?? "nearest"),
            mipmapFilter: String(descriptor.mipmapFilter ?? "nearest"),
            lodMinClamp: Number(descriptor.lodMinClamp ?? 0),
            lodMaxClamp: Number(descriptor.lodMaxClamp ?? 32),
            compare: descriptor.compare === undefined ? null : String(descriptor.compare),
            maxAnisotropy: Number(descriptor.maxAnisotropy ?? 1) >>> 0,
        };
        return new GPUSampler(
            construct,
            this,
            call("gpuCreateSampler", this.__id, JSON.stringify(normalized)),
            descriptor.label,
        );
    }
    createShaderModule(descriptor) {
        if (!descriptor || typeof descriptor.code !== "string") throw new TypeError("GPUShaderModuleDescriptor.code is required");
        return new GPUShaderModule(
            construct,
            this,
            call("gpuCreateShaderModule", this.__id, descriptor.code),
            descriptor.label,
        );
    }
    createQuerySet(descriptor) {
        if (!descriptor || typeof descriptor !== "object") throw new TypeError("GPUQuerySetDescriptor is required");
        const normalized = {
            type: String(descriptor.type),
            count: Number(descriptor.count) >>> 0,
            label: descriptor.label,
        };
        if (normalized.type !== "occlusion" && normalized.type !== "timestamp") throw new TypeError("Invalid GPU query type");
        if (normalized.type === "timestamp" && !this.features.has("timestamp-query")) {
            throw new DOMException("The timestamp-query feature is not enabled", "NotSupportedError");
        }
        return new GPUQuerySet(
            construct,
            this,
            call("gpuCreateQuerySet", this.__id, normalized.type, normalized.count),
            normalized,
        );
    }
    createBindGroupLayout(descriptor) {
        if (!descriptor || typeof descriptor !== "object") throw new TypeError("GPUBindGroupLayoutDescriptor is required");
        const entries = Array.from(descriptor.entries ?? [], entry => {
            if (!entry || typeof entry !== "object") throw new TypeError("Invalid GPUBindGroupLayoutEntry");
            if (entry.count !== undefined) throw new DOMException("Binding arrays are not implemented", "NotSupportedError");
            const layouts = ["buffer", "sampler", "texture", "storageTexture"].filter(key => entry[key] !== undefined);
            if (entry.externalTexture !== undefined) throw new DOMException("External textures are not implemented", "NotSupportedError");
            if (layouts.length !== 1) throw new TypeError("A bind group layout entry must specify exactly one resource layout");
            const common = { binding: Number(entry.binding) >>> 0, visibility: Number(entry.visibility) >>> 0 };
            if (entry.buffer !== undefined) return {
                kind: "buffer", ...common,
                ty: String(entry.buffer.type ?? "uniform"),
                has_dynamic_offset: Boolean(entry.buffer.hasDynamicOffset),
                min_binding_size: Number(entry.buffer.minBindingSize ?? 0) || null,
            };
            if (entry.sampler !== undefined) return {
                kind: "sampler", ...common, ty: String(entry.sampler.type ?? "filtering"),
            };
            if (entry.texture !== undefined) return {
                kind: "texture", ...common,
                sample_type: String(entry.texture.sampleType ?? "float"),
                view_dimension: String(entry.texture.viewDimension ?? "2d"),
                multisampled: Boolean(entry.texture.multisampled),
            };
            return {
                kind: "storageTexture", ...common,
                access: String(entry.storageTexture.access ?? "write-only"),
                format: String(entry.storageTexture.format),
                view_dimension: String(entry.storageTexture.viewDimension ?? "2d"),
            };
        });
        return new GPUBindGroupLayout(
            construct,
            this,
            call("gpuCreateBindGroupLayout", this.__id, JSON.stringify(entries)),
            descriptor.label,
        );
    }
    createPipelineLayout(descriptor) {
        if (!descriptor || typeof descriptor !== "object") throw new TypeError("GPUPipelineLayoutDescriptor is required");
        const layouts = Array.from(descriptor.bindGroupLayouts ?? []);
        for (const layout of layouts) {
            if (!(layout instanceof GPUBindGroupLayout) || layout.__device !== this) throw new TypeError("Invalid GPUBindGroupLayout");
        }
        return new GPUPipelineLayout(
            construct,
            this,
            call("gpuCreatePipelineLayout", this.__id, JSON.stringify(layouts.map(layout => layout.__id)),
                Number(descriptor.immediateSize ?? 0) >>> 0),
            descriptor.label,
        );
    }
    createComputePipeline(descriptor) {
        if (!descriptor || !descriptor.compute) throw new TypeError("GPUComputePipelineDescriptor is required");
        const layout = descriptor.layout;
        if (layout !== "auto" && (!(layout instanceof GPUPipelineLayout) || layout.__device !== this)) throw new TypeError("Invalid GPUPipelineLayout");
        const module = descriptor.compute.module;
        if (!(module instanceof GPUShaderModule) || module.__device !== this) throw new TypeError("Invalid GPUShaderModule");
        const entryPoint = descriptor.compute.entryPoint ?? "main";
        return new GPUComputePipeline(
            construct,
            this,
            call("gpuCreateComputePipeline", this.__id, module.__id, String(entryPoint), layout === "auto" ? 0 : layout.__id),
            descriptor.label,
        );
    }
    createComputePipelineAsync(descriptor) {
        return this.__createPipelineAsync(() => this.createComputePipeline(descriptor));
    }
    createRenderPipeline(descriptor) {
        if (!descriptor || !descriptor.vertex || !descriptor.fragment) throw new TypeError("GPURenderPipelineDescriptor is required");
        const layout = descriptor.layout;
        if (layout !== "auto" && (!(layout instanceof GPUPipelineLayout) || layout.__device !== this)) throw new TypeError("Invalid GPUPipelineLayout");
        const vertex = descriptor.vertex;
        const fragment = descriptor.fragment;
        if (!(vertex.module instanceof GPUShaderModule) || vertex.module.__device !== this ||
            !(fragment.module instanceof GPUShaderModule) || fragment.module.__device !== this) {
            throw new TypeError("Invalid GPUShaderModule");
        }
        const vertexBuffers = Array.from(vertex.buffers ?? [], layout => ({
            array_stride: Number(layout.arrayStride),
            step_mode: String(layout.stepMode ?? "vertex"),
            attributes: Array.from(layout.attributes ?? [], attribute => ({
                format: String(attribute.format),
                offset: Number(attribute.offset),
                shader_location: Number(attribute.shaderLocation) >>> 0,
            })),
        }));
        const targets = Array.from(fragment.targets ?? [], target => {
            if (target === null) return null;
            if (!target || target.format === undefined) throw new TypeError("Invalid GPU color target");
            const component = value => ({
                src_factor: String(value?.srcFactor ?? "one"),
                dst_factor: String(value?.dstFactor ?? "zero"),
                operation: String(value?.operation ?? "add"),
            });
            return {
                format: String(target.format),
                blend: target.blend ? { color: component(target.blend.color), alpha: component(target.blend.alpha) } : null,
                write_mask: Number(target.writeMask ?? 0xF) >>> 0,
            };
        });
        if (!targets.length) throw new TypeError("At least one fragment target is required");
        const stencilFace = value => ({
            compare: String(value?.compare ?? "always"),
            fail_op: String(value?.failOp ?? "keep"),
            depth_fail_op: String(value?.depthFailOp ?? "keep"),
            pass_op: String(value?.passOp ?? "keep"),
        });
        const depthStencil = descriptor.depthStencil ? {
            format: String(descriptor.depthStencil.format),
            depth_write_enabled: Boolean(descriptor.depthStencil.depthWriteEnabled),
            depth_compare: String(descriptor.depthStencil.depthCompare ?? "always"),
            stencil_front: stencilFace(descriptor.depthStencil.stencilFront),
            stencil_back: stencilFace(descriptor.depthStencil.stencilBack),
            stencil_read_mask: Number(descriptor.depthStencil.stencilReadMask ?? 0xFFFFFFFF) >>> 0,
            stencil_write_mask: Number(descriptor.depthStencil.stencilWriteMask ?? 0xFFFFFFFF) >>> 0,
            depth_bias: Number(descriptor.depthStencil.depthBias ?? 0) | 0,
            depth_bias_slope_scale: Number(descriptor.depthStencil.depthBiasSlopeScale ?? 0),
            depth_bias_clamp: Number(descriptor.depthStencil.depthBiasClamp ?? 0),
        } : null;
        const multisample = {
            count: Number(descriptor.multisample?.count ?? 1) >>> 0,
            mask: Number(descriptor.multisample?.mask ?? 0xFFFFFFFF) >>> 0,
            alpha_to_coverage_enabled: Boolean(descriptor.multisample?.alphaToCoverageEnabled),
        };
        const primitive = {
            topology: String(descriptor.primitive?.topology ?? "triangle-list"),
            strip_index_format: descriptor.primitive?.stripIndexFormat === undefined
                ? null
                : String(descriptor.primitive.stripIndexFormat),
            front_face: String(descriptor.primitive?.frontFace ?? "ccw"),
            cull_mode: String(descriptor.primitive?.cullMode ?? "none"),
            unclipped_depth: Boolean(descriptor.primitive?.unclippedDepth),
        };
        return new GPURenderPipeline(
            construct,
            this,
            call(
                "gpuCreateRenderPipeline",
                this.__id,
                vertex.module.__id,
                String(vertex.entryPoint ?? "main"),
                fragment.module.__id,
                String(fragment.entryPoint ?? "main"),
                JSON.stringify(vertexBuffers),
                JSON.stringify(targets),
                JSON.stringify(primitive),
                JSON.stringify(depthStencil),
                layout === "auto" ? 0 : layout.__id,
                JSON.stringify(multisample),
            ),
            descriptor.label,
        );
    }
    createRenderPipelineAsync(descriptor) {
        return this.__createPipelineAsync(() => this.createRenderPipeline(descriptor));
    }
    __createPipelineAsync(create) {
        this.pushErrorScope("internal");
        this.pushErrorScope("validation");
        let pipeline;
        let thrown;
        try {
            pipeline = create();
        } catch (error) {
            thrown = error;
        }
        const validation = this.popErrorScope();
        const internal = this.popErrorScope();
        return Promise.all([validation, internal]).then(([validationError, internalError]) => {
            if (thrown !== undefined) throw thrown;
            const error = validationError ?? internalError;
            if (error !== null) {
                throw new GPUPipelineError(error.message, {
                    reason: validationError !== null ? "validation" : "internal",
                });
            }
            return pipeline;
        });
    }
    createBindGroup(descriptor) {
        if (!descriptor || !(descriptor.layout instanceof GPUBindGroupLayout) || descriptor.layout.__device !== this) {
            throw new TypeError("Invalid GPUBindGroupLayout");
        }
        const entries = Array.from(descriptor.entries ?? [], entry => {
            const resource = entry.resource instanceof GPUBuffer ? { buffer: entry.resource } : entry.resource;
            const binding = Number(entry.binding) >>> 0;
            if (resource instanceof GPUSampler) {
                if (resource.__device !== this) throw new TypeError("Invalid GPUSampler");
                return { kind: "sampler", binding, resource: resource.__id };
            }
            if (resource instanceof GPUTextureView) {
                if (resource.__device !== this) throw new TypeError("Invalid GPUTextureView");
                return { kind: "textureView", binding, resource: resource.__id };
            }
            if (!resource || !(resource.buffer instanceof GPUBuffer) || resource.buffer.__device !== this) throw new TypeError("Invalid GPU binding resource");
            return {
                kind: "buffer", binding, resource: resource.buffer.__id,
                offset: Number(resource.offset ?? 0),
                size: resource.size === undefined ? null : Number(resource.size),
            };
        });
        return new GPUBindGroup(
            construct,
            this,
            call("gpuCreateBindGroup", this.__id, descriptor.layout.__id, JSON.stringify(entries)),
            descriptor.label,
        );
    }
    destroy() {
        if (this.__destroyed) return;
        this.__destroyed = true;
        call("gpuDestroyDevice", this.__id);
    }
    pushErrorScope(filter) {
        filter = String(filter);
        if (!['validation', 'out-of-memory', 'internal'].includes(filter)) throw new TypeError("Invalid GPUErrorFilter");
        call("gpuPushErrorScope", this.__id, filter);
    }
    popErrorScope() {
        try {
            const encoded = call("gpuPopErrorScope", this.__id);
            if (encoded === null) return Promise.resolve(null);
            const error = JSON.parse(encoded);
            const constructor = error.kind === "validation" ? GPUValidationError
                : error.kind === "out-of-memory" ? GPUOutOfMemoryError
                : GPUInternalError;
            return Promise.resolve(new constructor(error.message));
        } catch (error) {
            return Promise.reject(new DOMException(error.message, "OperationError"));
        }
    }
}

class GPUAdapter {
    constructor(token, id, metadata) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __id: { value: id },
            features: { value: new GPUSupportedFeatures(construct, metadata.features), enumerable: true },
            limits: { value: new GPUSupportedLimits(construct, metadata.limits), enumerable: true },
            info: { value: new GPUAdapterInfo(construct, metadata.info), enumerable: true },
            isFallbackAdapter: { value: Boolean(metadata.isFallbackAdapter), enumerable: true },
        });
    }
    requestDevice(descriptor = {}) {
        try {
            descriptor ??= {};
            const requiredFeatures = Array.from(descriptor.requiredFeatures ?? [], String);
            if (requiredFeatures.some(feature => !this.features.has(feature))) {
                throw new DOMException("A required GPU feature is unavailable", "NotSupportedError");
            }
            const requiredLimits = {};
            for (const [name, rawValue] of Object.entries(descriptor.requiredLimits ?? {})) {
                if (!Object.hasOwn(this.limits, name)) {
                    throw new DOMException(`Unknown required GPU limit: ${name}`, "OperationError");
                }
                const value = Number(rawValue);
                if (!Number.isSafeInteger(value) || value < 0) {
                    throw new TypeError(`Required GPU limit ${name} must be a non-negative safe integer`);
                }
                const supported = Number(this.limits[name]);
                const unsupported = minimumGpuLimits.has(name)
                    ? value < supported
                    : value > supported;
                if (unsupported) {
                    throw new DOMException(
                        `Required GPU limit ${name} (${value}) exceeds adapter support (${supported})`,
                        "OperationError",
                    );
                }
                requiredLimits[name] = value;
            }
            const label = String(descriptor.label ?? "");
            const metadata = JSON.parse(call(
                "gpuRequestDevice",
                this.__id,
                JSON.stringify(requiredFeatures),
                JSON.stringify(requiredLimits),
                label,
            ));
            metadata.limits.maxBindGroupsPlusVertexBuffers = Math.min(
                metadata.limits.maxBindGroupsPlusVertexBuffers,
                this.limits.maxBindGroupsPlusVertexBuffers,
            );
            return Promise.resolve(new GPUDevice(construct, metadata.id, metadata));
        } catch (error) {
            return Promise.reject(error instanceof DOMException || error instanceof TypeError
                ? error
                : new DOMException(error.message, "OperationError"));
        }
    }
    requestAdapterInfo() { return Promise.resolve(this.info); }
}

class GPU {
    constructor(token) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "wgslLanguageFeatures", {
            value: new WGSLLanguageFeatures(construct, []),
            enumerable: true,
        });
    }
    requestAdapter(options = {}) {
        const preference = options.powerPreference === "low-power" || options.powerPreference === "high-performance"
            ? options.powerPreference : "none";
        try {
            const encoded = call("gpuRequestAdapter", preference, Boolean(options.forceFallbackAdapter));
            if (encoded === null) return Promise.resolve(null);
            const result = JSON.parse(encoded);
            applyAdapterPersona(result.metadata);
            return Promise.resolve(new GPUAdapter(construct, result.id, result.metadata));
        } catch (error) {
            return Promise.reject(error);
        }
    }
    getPreferredCanvasFormat() { return "bgra8unorm"; }
}

const GPUBufferUsage = Object.freeze({
    MAP_READ: 1, MAP_WRITE: 2, COPY_SRC: 4, COPY_DST: 8, INDEX: 16,
    VERTEX: 32, UNIFORM: 64, STORAGE: 128, INDIRECT: 256, QUERY_RESOLVE: 512,
});
const GPUMapMode = Object.freeze({ READ: 1, WRITE: 2 });
const GPUShaderStage = Object.freeze({ VERTEX: 1, FRAGMENT: 2, COMPUTE: 4 });
const GPUTextureUsage = Object.freeze({
    COPY_SRC: 1, COPY_DST: 2, TEXTURE_BINDING: 4, STORAGE_BINDING: 8, RENDER_ATTACHMENT: 16,
});

Object.defineProperty(navigator, "gpu", { value: new GPU(construct), enumerable: true, configurable: true });
Object.defineProperty(globalThis, "__brimpSetGpuPersona", {
    value(value) { graphicsPersona = Object.freeze({ ...value }); },
    configurable: true,
});
Object.defineProperty(globalThis, "__brimpCreateWebGPUContext", {
    value(canvas) {
        return call("gpuCanvasAcquire", canvas, canvas.width, canvas.height)
            ? new GPUCanvasContext(construct, canvas)
            : null;
    },
    configurable: true,
});
Object.defineProperties(globalThis, {
    GPUError: { value: GPUError, writable: true, configurable: true },
    GPUValidationError: { value: GPUValidationError, writable: true, configurable: true },
    GPUOutOfMemoryError: { value: GPUOutOfMemoryError, writable: true, configurable: true },
    GPUInternalError: { value: GPUInternalError, writable: true, configurable: true },
    GPUUncapturedErrorEvent: { value: GPUUncapturedErrorEvent, writable: true, configurable: true },
    GPUDeviceLostInfo: { value: GPUDeviceLostInfo, writable: true, configurable: true },
    GPUPipelineError: { value: GPUPipelineError, writable: true, configurable: true },
    GPU: { value: GPU, writable: true, configurable: true },
    GPUAdapter: { value: GPUAdapter, writable: true, configurable: true },
    GPUAdapterInfo: { value: GPUAdapterInfo, writable: true, configurable: true },
    GPUCompilationInfo: { value: GPUCompilationInfo, writable: true, configurable: true },
    GPUCompilationMessage: { value: GPUCompilationMessage, writable: true, configurable: true },
    GPUDevice: { value: GPUDevice, writable: true, configurable: true },
    GPUQueue: { value: GPUQueue, writable: true, configurable: true },
    GPUCanvasContext: { value: GPUCanvasContext, writable: true, configurable: true },
    GPUBuffer: { value: GPUBuffer, writable: true, configurable: true },
    GPUTexture: { value: GPUTexture, writable: true, configurable: true },
    GPUTextureView: { value: GPUTextureView, writable: true, configurable: true },
    GPUSampler: { value: GPUSampler, writable: true, configurable: true },
    GPUQuerySet: { value: GPUQuerySet, writable: true, configurable: true },
    GPUShaderModule: { value: GPUShaderModule, writable: true, configurable: true },
    GPUComputePipeline: { value: GPUComputePipeline, writable: true, configurable: true },
    GPURenderPipeline: { value: GPURenderPipeline, writable: true, configurable: true },
    GPUBindGroupLayout: { value: GPUBindGroupLayout, writable: true, configurable: true },
    GPUPipelineLayout: { value: GPUPipelineLayout, writable: true, configurable: true },
    GPUBindGroup: { value: GPUBindGroup, writable: true, configurable: true },
    GPUComputePassEncoder: { value: GPUComputePassEncoder, writable: true, configurable: true },
    GPURenderPassEncoder: { value: GPURenderPassEncoder, writable: true, configurable: true },
    GPURenderBundleEncoder: { value: GPURenderBundleEncoder, writable: true, configurable: true },
    GPURenderBundle: { value: GPURenderBundle, writable: true, configurable: true },
    GPUCommandEncoder: { value: GPUCommandEncoder, writable: true, configurable: true },
    GPUCommandBuffer: { value: GPUCommandBuffer, writable: true, configurable: true },
    GPUSupportedLimits: { value: GPUSupportedLimits, writable: true, configurable: true },
    GPUSupportedFeatures: { value: GPUSupportedFeatures, writable: true, configurable: true },
    WGSLLanguageFeatures: { value: WGSLLanguageFeatures, writable: true, configurable: true },
    GPUBufferUsage: { value: GPUBufferUsage, configurable: true },
    GPUTextureUsage: { value: GPUTextureUsage, configurable: true },
    GPUShaderStage: { value: GPUShaderStage, configurable: true },
    GPUMapMode: { value: GPUMapMode, configurable: true },
});

for (const constructor of [GPUError, GPUValidationError, GPUOutOfMemoryError, GPUInternalError, GPUUncapturedErrorEvent, GPUDeviceLostInfo, GPUPipelineError, GPU, GPUAdapter, GPUAdapterInfo, GPUCompilationInfo, GPUCompilationMessage, GPUDevice, GPUQueue, GPUCanvasContext, GPUBuffer, GPUTexture, GPUTextureView, GPUSampler, GPUQuerySet, GPUShaderModule, GPUComputePipeline, GPURenderPipeline, GPUBindGroupLayout, GPUPipelineLayout, GPUBindGroup, GPUComputePassEncoder, GPURenderPassEncoder, GPURenderBundleEncoder, GPURenderBundle, GPUCommandEncoder, GPUCommandBuffer, GPUSupportedLimits, GPUSupportedFeatures, WGSLLanguageFeatures]) {
    globalThis.__brimpMarkWebBuiltin?.(constructor);
    for (const key of Reflect.ownKeys(constructor.prototype)) {
        if (key === "constructor" || String(key).startsWith("__")) continue;
        const descriptor = Object.getOwnPropertyDescriptor(constructor.prototype, key);
        if (typeof descriptor?.value === "function") {
            const name = key === Symbol.iterator ? "values" : String(key);
            globalThis.__brimpMarkWebBuiltin?.(descriptor.value, `function ${name}() { [native code] }`);
        }
        if (typeof descriptor?.get === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.get, `function get ${String(key)}() { [native code] }`);
    }
}
})();
