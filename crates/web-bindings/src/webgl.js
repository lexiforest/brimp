(() => {
"use strict";

const host = globalThis.__brimpWebGlHost;
const markTrustedEvent = globalThis.__brimpMarkTrustedEvent;
const construct = Symbol("WebGL construction");
let graphicsPersona = null;
const contextsByCanvas = new WeakMap();
const call = (operation, canvas, ...arguments_) => {
    const context = contextsByCanvas.get(canvas);
    if (context?.__contextLost && operation !== "webglRestoreContext" && operation !== "webglLoseContext") {
        return undefined;
    }
    return host(operation, canvas, ...arguments_);
};

class WebGLObject {
    constructor(token, context, id) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            __id: { value: id, writable: true },
        });
        context.__objects.add(this);
    }
}

class WebGLShader extends WebGLObject {
    constructor(token, context, id, type) {
        super(token, context, id);
        Object.defineProperty(this, "__source", { value: "", writable: true });
        Object.defineProperty(this, "__type", { value: type });
    }
}
class WebGLProgram extends WebGLObject {
    constructor(token, context, id) {
        super(token, context, id);
        Object.defineProperty(this, "__attachedShaders", { value: new Set() });
    }
}
class WebGLBuffer extends WebGLObject {}
class WebGLTexture extends WebGLObject {}
class WebGLFramebuffer extends WebGLObject {}
class WebGLRenderbuffer extends WebGLObject {}
class WebGLSampler extends WebGLObject {}
class WebGLQuery extends WebGLObject {
    constructor(token, context, id) {
        super(token, context, id);
        Object.defineProperty(this, "__target", { value: null, writable: true });
    }
}
class WebGLTimerQueryEXT extends WebGLObject {
    constructor(token, context, id) {
        super(token, context, id);
        Object.defineProperty(this, "__target", { value: null, writable: true });
    }
}
class WebGLSync extends WebGLObject {}
class WebGLTransformFeedback extends WebGLObject {}
class WebGLActiveInfo {
    constructor(token, name, size, type) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            name: { value: name, enumerable: true },
            size: { value: size, enumerable: true },
            type: { value: type, enumerable: true },
        });
    }
}
class WebGLShaderPrecisionFormat {
    constructor(token, rangeMin, rangeMax, precision) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            rangeMin: { value: rangeMin, enumerable: true },
            rangeMax: { value: rangeMax, enumerable: true },
            precision: { value: precision, enumerable: true },
        });
    }
}
class WebGLUniformLocation extends WebGLObject {
    constructor(token, context, id, name) {
        super(token, context, id);
        Object.defineProperty(this, "__name", { value: name });
    }
}
class WebGLVertexArrayObject extends WebGLObject {}

class WebGLContextEvent extends Event {
    constructor(type, options = {}) {
        super(type, options);
        Object.defineProperty(this, "__statusMessage", {
            value: options.statusMessage === undefined ? "" : String(options.statusMessage),
        });
    }
    get statusMessage() { return this.__statusMessage; }
}

class WebGLLoseContext {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__context", { value: context });
    }
    loseContext() { loseContext(this.__context); }
    restoreContext() { restoreContext(this.__context); }
}

class WEBGLDebugShaders {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__context", { value: context });
    }
    getTranslatedShaderSource(shader) {
        if (this.__context.__contextLost) return "";
        return call(
            "webglGetTranslatedShaderSource",
            this.__context.canvas,
            objectId(this.__context, shader, WebGLShader),
        );
    }
}

class EXTClipControl {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            LOWER_LEFT_EXT: { value: 0x8CA1, enumerable: true },
            UPPER_LEFT_EXT: { value: 0x8CA2, enumerable: true },
            NEGATIVE_ONE_TO_ONE_EXT: { value: 0x935E, enumerable: true },
            ZERO_TO_ONE_EXT: { value: 0x935F, enumerable: true },
            CLIP_ORIGIN_EXT: { value: 0x935C, enumerable: true },
            CLIP_DEPTH_MODE_EXT: { value: 0x935D, enumerable: true },
        });
    }
    clipControlEXT(origin, depth) {
        call(
            "webglClipControl",
            this.__context.canvas,
            Number(origin) >>> 0,
            Number(depth) >>> 0,
        );
    }
}

class EXTPolygonOffsetClamp {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            POLYGON_OFFSET_CLAMP_EXT: { value: 0x8E1B, enumerable: true },
        });
    }
    polygonOffsetClampEXT(factor, units, clamp) {
        call(
            "webglPolygonOffsetClamp",
            this.__context.canvas,
            Number(factor),
            Number(units),
            Number(clamp),
        );
    }
}

class WEBGLProvokingVertex {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            FIRST_VERTEX_CONVENTION_WEBGL: { value: 0x8E4D, enumerable: true },
            LAST_VERTEX_CONVENTION_WEBGL: { value: 0x8E4E, enumerable: true },
            PROVOKING_VERTEX_WEBGL: { value: 0x8E4F, enumerable: true },
        });
    }
    provokingVertexWEBGL(mode) {
        call("webglProvokingVertex", this.__context.canvas, Number(mode) >>> 0);
    }
}

class WEBGLPolygonMode {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            POLYGON_MODE_WEBGL: { value: 0x0B40, enumerable: true },
            POLYGON_OFFSET_LINE_WEBGL: { value: 0x2A02, enumerable: true },
            LINE_WEBGL: { value: 0x1B01, enumerable: true },
            FILL_WEBGL: { value: 0x1B02, enumerable: true },
        });
    }
    polygonModeWEBGL(face, mode) {
        call(
            "webglPolygonMode",
            this.__context.canvas,
            Number(face) >>> 0,
            Number(mode) >>> 0,
        );
    }
}

class OESVertexArrayObject {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            VERTEX_ARRAY_BINDING_OES: { value: 0x85B5, enumerable: true },
        });
    }
    createVertexArrayOES() {
        if (this.__context.__contextLost) return null;
        return new WebGLVertexArrayObject(
            construct,
            this.__context,
            call("webglCreateVertexArray", this.__context.canvas),
        );
    }
    deleteVertexArrayOES(array) {
        if (array === null || array?.__id === 0) return;
        call(
            "webglDeleteVertexArray",
            this.__context.canvas,
            objectId(this.__context, array, WebGLVertexArrayObject),
        );
        if (this.__context.__vertexArray === array) this.__context.__vertexArray = null;
        array.__id = 0;
    }
    isVertexArrayOES(array) {
        return array instanceof WebGLVertexArrayObject
            && array.__context === this.__context && array.__id !== 0;
    }
    bindVertexArrayOES(array) {
        call(
            "webglBindVertexArray",
            this.__context.canvas,
            objectId(this.__context, array, WebGLVertexArrayObject, true),
        );
        this.__context.__vertexArray = array;
    }
}

class ANGLEInstancedArrays {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE: { value: 0x88FE, enumerable: true },
        });
    }
    drawArraysInstancedANGLE(mode, first, count, instanceCount) {
        call(
            "webglDrawArraysInstanced",
            this.__context.canvas,
            Number(mode) >>> 0,
            Number(first) | 0,
            Number(count) | 0,
            Number(instanceCount) | 0,
        );
    }
    drawElementsInstancedANGLE(mode, count, type, offset, instanceCount) {
        call(
            "webglDrawElementsInstanced",
            this.__context.canvas,
            Number(mode) >>> 0,
            Number(count) | 0,
            Number(type) >>> 0,
            Number(offset) | 0,
            Number(instanceCount) | 0,
        );
    }
    vertexAttribDivisorANGLE(index, divisor) {
        call(
            "webglVertexAttribDivisor",
            this.__context.canvas,
            Number(index) >>> 0,
            Number(divisor) >>> 0,
        );
    }
}

class WEBGLDrawBuffers {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            MAX_DRAW_BUFFERS_WEBGL: { value: 0x8824, enumerable: true },
            MAX_COLOR_ATTACHMENTS_WEBGL: { value: 0x8CDF, enumerable: true },
        });
        for (let index = 0; index < 16; index += 1) {
            Object.defineProperty(this, `DRAW_BUFFER${index}_WEBGL`, {
                value: 0x8825 + index,
                enumerable: true,
            });
            Object.defineProperty(this, `COLOR_ATTACHMENT${index}_WEBGL`, {
                value: 0x8CE0 + index,
                enumerable: true,
            });
        }
    }
    drawBuffersWEBGL(buffers) {
        call(
            "webglDrawBuffers",
            this.__context.canvas,
            new Uint32Array(Array.from(buffers, value => Number(value) >>> 0)),
        );
    }
}

class WEBGLCompressedTextureASTC {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__context", { value: context });
        const blockSizes = [
            "4x4", "5x4", "5x5", "6x5", "6x6", "8x5", "8x6",
            "8x8", "10x5", "10x6", "10x8", "10x10", "12x10", "12x12",
        ];
        for (let index = 0; index < blockSizes.length; index += 1) {
            Object.defineProperty(this, `COMPRESSED_RGBA_ASTC_${blockSizes[index]}_KHR`, {
                value: 0x93B0 + index,
                enumerable: true,
            });
            Object.defineProperty(this, `COMPRESSED_SRGB8_ALPHA8_ASTC_${blockSizes[index]}_KHR`, {
                value: 0x93D0 + index,
                enumerable: true,
            });
        }
    }
    getSupportedProfiles() { return ["ldr"]; }
}

class EXTDisjointTimerQuery {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            QUERY_COUNTER_BITS_EXT: { value: 0x8864, enumerable: true },
            CURRENT_QUERY_EXT: { value: 0x8865, enumerable: true },
            QUERY_RESULT_EXT: { value: 0x8866, enumerable: true },
            QUERY_RESULT_AVAILABLE_EXT: { value: 0x8867, enumerable: true },
            TIME_ELAPSED_EXT: { value: 0x88BF, enumerable: true },
            TIMESTAMP_EXT: { value: 0x8E28, enumerable: true },
            GPU_DISJOINT_EXT: { value: 0x8FBB, enumerable: true },
        });
    }
    createQueryEXT() {
        if (this.__context.__contextLost) return null;
        return new WebGLTimerQueryEXT(
            construct,
            this.__context,
            call("webglCreateQuery", this.__context.canvas),
        );
    }
    deleteQueryEXT(query) {
        if (query === null || query?.__id === 0) return;
        const id = objectId(this.__context, query, WebGLTimerQueryEXT);
        for (const [target, active] of this.__context.__queries) {
            if (active !== query) continue;
            call("webglEndQuery", this.__context.canvas, target);
            this.__context.__queries.set(target, null);
        }
        call("webglDeleteQuery", this.__context.canvas, id);
        query.__id = 0;
    }
    isQueryEXT(query) {
        return query instanceof WebGLTimerQueryEXT
            && query.__context === this.__context && query.__id !== 0;
    }
    beginQueryEXT(target, query) {
        target = Number(target) >>> 0;
        call("webglBeginQuery", this.__context.canvas, target,
            objectId(this.__context, query, WebGLTimerQueryEXT));
        this.__context.__queries.set(target, query);
        query.__target = target;
    }
    endQueryEXT(target) {
        target = Number(target) >>> 0;
        call("webglEndQuery", this.__context.canvas, target);
        this.__context.__queries.set(target, null);
    }
    queryCounterEXT(query, target) {
        target = Number(target) >>> 0;
        call("webglQueryCounter", this.__context.canvas,
            objectId(this.__context, query, WebGLTimerQueryEXT), target);
        query.__target = target;
    }
    getQueryEXT(target, parameter) {
        target = Number(target) >>> 0;
        parameter = Number(parameter) >>> 0;
        if (parameter === this.CURRENT_QUERY_EXT) {
            return target === this.TIMESTAMP_EXT
                ? null : this.__context.__queries.get(target) ?? null;
        }
        if (parameter === this.QUERY_COUNTER_BITS_EXT) {
            return call("webglGetQueryCounterBits", this.__context.canvas, target);
        }
        return null;
    }
    getQueryObjectEXT(query, parameter) {
        parameter = Number(parameter) >>> 0;
        const id = objectId(this.__context, query, WebGLTimerQueryEXT);
        if (parameter === this.QUERY_RESULT_AVAILABLE_EXT) {
            return Boolean(call("webglGetQueryParameter", this.__context.canvas, id, parameter));
        }
        if (parameter === this.QUERY_RESULT_EXT) {
            return call("webglGetQueryParameter64", this.__context.canvas, id, parameter);
        }
        return null;
    }
}

class EXTDisjointTimerQueryWebGL2 {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context },
            QUERY_COUNTER_BITS_EXT: { value: 0x8864, enumerable: true },
            TIME_ELAPSED_EXT: { value: 0x88BF, enumerable: true },
            TIMESTAMP_EXT: { value: 0x8E28, enumerable: true },
            GPU_DISJOINT_EXT: { value: 0x8FBB, enumerable: true },
        });
    }
    queryCounterEXT(query, target) {
        target = Number(target) >>> 0;
        call("webglQueryCounter", this.__context.canvas,
            objectId(this.__context, query, WebGLQuery), target);
        query.__target = target;
    }
}

class OESDrawBuffersIndexed {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__context", { value: context });
    }
    enableiOES(target, index) {
        call("webglSetEnabledIndexed", this.__context.canvas,
            Number(target) >>> 0, Number(index) >>> 0, true);
    }
    disableiOES(target, index) {
        call("webglSetEnabledIndexed", this.__context.canvas,
            Number(target) >>> 0, Number(index) >>> 0, false);
    }
    blendEquationiOES(index, mode) {
        call("webglBlendEquationIndexed", this.__context.canvas,
            Number(index) >>> 0, Number(mode) >>> 0);
    }
    blendEquationSeparateiOES(index, modeRGB, modeAlpha) {
        call("webglBlendEquationSeparateIndexed", this.__context.canvas,
            Number(index) >>> 0, Number(modeRGB) >>> 0, Number(modeAlpha) >>> 0);
    }
    blendFunciOES(index, source, destination) {
        call("webglBlendFuncIndexed", this.__context.canvas,
            Number(index) >>> 0, Number(source) >>> 0, Number(destination) >>> 0);
    }
    blendFuncSeparateiOES(index, sourceRGB, destinationRGB, sourceAlpha, destinationAlpha) {
        call("webglBlendFuncSeparateIndexed", this.__context.canvas,
            Number(index) >>> 0, Number(sourceRGB) >>> 0, Number(destinationRGB) >>> 0,
            Number(sourceAlpha) >>> 0, Number(destinationAlpha) >>> 0);
    }
    colorMaskiOES(index, red, green, blue, alpha) {
        call("webglColorMaskIndexed", this.__context.canvas,
            Number(index) >>> 0, Boolean(red), Boolean(green), Boolean(blue), Boolean(alpha));
    }
}

function multiDrawCount(context, value) {
    const count = Number(value) | 0;
    if (count < 0) {
        context.__errors.push(context.INVALID_VALUE);
        return null;
    }
    return count;
}

function multiDrawValues(context, value, offset, count, name) {
    let values;
    if (value instanceof Int32Array) {
        values = value;
    } else {
        if (value === null || value === undefined
            || typeof value[Symbol.iterator] !== "function") {
            throw new TypeError(`${name} must be an Int32Array or sequence`);
        }
        values = Int32Array.from(value, item => Number(item) | 0);
    }
    offset = Number(offset) >>> 0;
    if (offset > values.length || count > values.length - offset) {
        context.__errors.push(context.INVALID_OPERATION);
        return null;
    }
    return values.subarray(offset, offset + count);
}

function multiDrawUnsignedValues(context, value, offset, count, name) {
    let values;
    if (value instanceof Uint32Array) {
        values = value;
    } else {
        if (value === null || value === undefined
            || typeof value[Symbol.iterator] !== "function") {
            throw new TypeError(`${name} must be a Uint32Array or sequence`);
        }
        values = Uint32Array.from(value, item => Number(item) >>> 0);
    }
    offset = Number(offset) >>> 0;
    if (offset > values.length || count > values.length - offset) {
        context.__errors.push(context.INVALID_OPERATION);
        return null;
    }
    return values.subarray(offset, offset + count);
}

class WEBGLMultiDraw {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__context", { value: context });
    }
    multiDrawArraysWEBGL(mode, firsts, firstsOffset, counts, countsOffset, drawcount) {
        if (this.__context.__contextLost) return;
        drawcount = multiDrawCount(this.__context, drawcount);
        if (drawcount === null) return;
        firsts = multiDrawValues(
            this.__context, firsts, firstsOffset, drawcount, "firstsList");
        counts = multiDrawValues(
            this.__context, counts, countsOffset, drawcount, "countsList");
        if (firsts === null || counts === null) return;
        call("webglMultiDrawArrays", this.__context.canvas, Number(mode) >>> 0,
            firsts, counts);
    }
    multiDrawElementsWEBGL(mode, counts, countsOffset, type, offsets, offsetsOffset, drawcount) {
        if (this.__context.__contextLost) return;
        drawcount = multiDrawCount(this.__context, drawcount);
        if (drawcount === null) return;
        counts = multiDrawValues(
            this.__context, counts, countsOffset, drawcount, "countsList");
        offsets = multiDrawValues(
            this.__context, offsets, offsetsOffset, drawcount, "offsetsList");
        if (counts === null || offsets === null) return;
        call("webglMultiDrawElements", this.__context.canvas, Number(mode) >>> 0,
            counts, Number(type) >>> 0, offsets);
    }
    multiDrawArraysInstancedWEBGL(mode, firsts, firstsOffset, counts, countsOffset,
        instanceCounts, instanceCountsOffset, drawcount) {
        if (this.__context.__contextLost) return;
        drawcount = multiDrawCount(this.__context, drawcount);
        if (drawcount === null) return;
        firsts = multiDrawValues(
            this.__context, firsts, firstsOffset, drawcount, "firstsList");
        counts = multiDrawValues(
            this.__context, counts, countsOffset, drawcount, "countsList");
        instanceCounts = multiDrawValues(this.__context, instanceCounts,
            instanceCountsOffset, drawcount, "instanceCountsList");
        if (firsts === null || counts === null || instanceCounts === null) return;
        call("webglMultiDrawArraysInstanced", this.__context.canvas, Number(mode) >>> 0,
            firsts, counts, instanceCounts);
    }
    multiDrawElementsInstancedWEBGL(mode, counts, countsOffset, type,
        offsets, offsetsOffset, instanceCounts, instanceCountsOffset, drawcount) {
        if (this.__context.__contextLost) return;
        drawcount = multiDrawCount(this.__context, drawcount);
        if (drawcount === null) return;
        counts = multiDrawValues(
            this.__context, counts, countsOffset, drawcount, "countsList");
        offsets = multiDrawValues(
            this.__context, offsets, offsetsOffset, drawcount, "offsetsList");
        instanceCounts = multiDrawValues(this.__context, instanceCounts,
            instanceCountsOffset, drawcount, "instanceCountsList");
        if (counts === null || offsets === null || instanceCounts === null) return;
        call("webglMultiDrawElementsInstanced", this.__context.canvas, Number(mode) >>> 0,
            counts, Number(type) >>> 0, offsets, instanceCounts);
    }
}

class WEBGLDrawInstancedBaseVertexBaseInstance {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__context", { value: context });
    }
    drawArraysInstancedBaseInstanceWEBGL(mode, first, count, instanceCount, baseInstance) {
        if (this.__context.__contextLost) return;
        call("webglDrawArraysInstancedBaseInstance", this.__context.canvas,
            Number(mode) >>> 0, Number(first) | 0, Number(count) | 0,
            Number(instanceCount) | 0, Number(baseInstance) >>> 0);
    }
    drawElementsInstancedBaseVertexBaseInstanceWEBGL(mode, count, type, offset,
        instanceCount, baseVertex, baseInstance) {
        if (this.__context.__contextLost) return;
        call("webglDrawElementsInstancedBaseVertexBaseInstance", this.__context.canvas,
            Number(mode) >>> 0, Number(count) | 0, Number(type) >>> 0,
            Number(offset) | 0, Number(instanceCount) | 0, Number(baseVertex) | 0,
            Number(baseInstance) >>> 0);
    }
}

class WEBGLMultiDrawInstancedBaseVertexBaseInstance {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__context", { value: context });
    }
    multiDrawArraysInstancedBaseInstanceWEBGL(mode, firsts, firstsOffset,
        counts, countsOffset, instanceCounts, instanceCountsOffset,
        baseInstances, baseInstancesOffset, drawcount) {
        if (this.__context.__contextLost) return;
        drawcount = multiDrawCount(this.__context, drawcount);
        if (drawcount === null) return;
        firsts = multiDrawValues(
            this.__context, firsts, firstsOffset, drawcount, "firstsList");
        counts = multiDrawValues(
            this.__context, counts, countsOffset, drawcount, "countsList");
        instanceCounts = multiDrawValues(this.__context, instanceCounts,
            instanceCountsOffset, drawcount, "instanceCountsList");
        baseInstances = multiDrawUnsignedValues(this.__context, baseInstances,
            baseInstancesOffset, drawcount, "baseInstancesList");
        if (firsts === null || counts === null || instanceCounts === null
            || baseInstances === null) return;
        call("webglMultiDrawArraysInstancedBaseInstance", this.__context.canvas,
            Number(mode) >>> 0, firsts, counts, instanceCounts, baseInstances);
    }
    multiDrawElementsInstancedBaseVertexBaseInstanceWEBGL(mode, counts, countsOffset,
        type, offsets, offsetsOffset, instanceCounts, instanceCountsOffset,
        baseVertices, baseVerticesOffset, baseInstances, baseInstancesOffset, drawcount) {
        if (this.__context.__contextLost) return;
        drawcount = multiDrawCount(this.__context, drawcount);
        if (drawcount === null) return;
        counts = multiDrawValues(
            this.__context, counts, countsOffset, drawcount, "countsList");
        offsets = multiDrawValues(
            this.__context, offsets, offsetsOffset, drawcount, "offsetsList");
        instanceCounts = multiDrawValues(this.__context, instanceCounts,
            instanceCountsOffset, drawcount, "instanceCountsList");
        baseVertices = multiDrawValues(this.__context, baseVertices,
            baseVerticesOffset, drawcount, "baseVerticesList");
        baseInstances = multiDrawUnsignedValues(this.__context, baseInstances,
            baseInstancesOffset, drawcount, "baseInstancesList");
        if (counts === null || offsets === null || instanceCounts === null
            || baseVertices === null || baseInstances === null) return;
        call("webglMultiDrawElementsInstancedBaseVertexBaseInstance", this.__context.canvas,
            Number(mode) >>> 0, counts, Number(type) >>> 0, offsets,
            instanceCounts, baseVertices, baseInstances);
    }
}

function compressedTextureExtension(context, constants) {
    for (const format of Object.values(constants)) context.__compressedTextureFormats.add(format);
    return Object.freeze(constants);
}

function resetContextBindings(context) {
    context.__activeTexture = context.TEXTURE0;
    context.__texture2D.clear();
    context.__textureCube.clear();
    context.__texture3D.clear();
    context.__texture2DArray.clear();
    context.__samplers.clear();
    context.__queries.clear();
    context.__errors.length = 0;
    context.__indexedBuffers.clear();
    context.__currentAttributes.clear();
    context.__transformFeedback = null;
    context.__transformFeedbackActive = false;
    context.__transformFeedbackPaused = false;
    context.__vertexArray = null;
    context.__framebuffer = null;
    context.__drawFramebuffer = null;
    context.__readFramebuffer = null;
    context.__renderbuffer = null;
    context.__arrayBuffer = null;
    context.__elementArrayBuffer = null;
    context.__uniformBuffer = null;
    context.__copyReadBuffer = null;
    context.__copyWriteBuffer = null;
    context.__pixelPackBuffer = null;
    context.__pixelUnpackBuffer = null;
    context.__currentProgram = null;
    context.__unpackAlignment = 4;
    context.__unpackFlipY = false;
    context.__unpackPremultiplyAlpha = false;
}

function loseContext(context) {
    if (context.__contextLost || !call("webglLoseContext", context.canvas)) return;
    context.__contextLost = true;
    context.__lostError = true;
    context.__restoreAllowed = false;
    context.__restorePending = false;
    for (const object of context.__objects) object.__id = 0;
    context.__objects.clear();
    resetContextBindings(context);
    setTimeout(() => {
        if (!context.__contextLost) return;
        const event = new WebGLContextEvent("webglcontextlost", {
            cancelable: true,
            statusMessage: "Context lost through WEBGL_lose_context",
        });
        markTrustedEvent(event);
        context.canvas.dispatchEvent(event);
        context.__restoreAllowed = event.defaultPrevented;
    }, 0);
}

function restoreContext(context) {
    if (!context.__contextLost || !context.__restoreAllowed || context.__restorePending) return;
    context.__restorePending = true;
    setTimeout(() => {
        context.__restorePending = false;
        if (!context.__contextLost || !context.__restoreAllowed) return;
        if (!call("webglRestoreContext", context.canvas, context.canvas.width, context.canvas.height, context.__version)) return;
        context.__contextLost = false;
        context.__restoreAllowed = false;
        context.__lostError = false;
        resetContextBindings(context);
        for (const canonical of context.__extensions.keys()) {
            if (canonical !== "WEBGL_debug_renderer_info") {
                call("webglEnableWebExtension", context.canvas, canonical);
            }
        }
        const event = new WebGLContextEvent("webglcontextrestored");
        markTrustedEvent(event);
        context.canvas.dispatchEvent(event);
    }, 0);
}

function objectId(context, value, constructor, nullable = false) {
    if (nullable && value === null) return 0;
    if (!(value instanceof constructor) || value.__context !== context || value.__id === 0) {
        throw new TypeError(`Expected ${constructor.name} from this context`);
    }
    return value.__id;
}

function objectById(context, id, constructor) {
    if (id === null || id === 0) return null;
    for (const object of context.__objects) {
        if (object instanceof constructor && object.__id === id) return object;
    }
    return null;
}

function bytes(value) {
    if (typeof value === "number") return new Uint8Array(Math.max(0, Number(value) >>> 0));
    if (value instanceof ArrayBuffer) return new Uint8Array(value);
    if (ArrayBuffer.isView(value)) {
        return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
    }
    throw new TypeError("bufferData expects a size, ArrayBuffer, or ArrayBufferView");
}

function bufferSourceBytes(context, value, sourceOffset = 0, sourceLength = 0) {
    if (context.__version !== 2 || sourceOffset === 0 && sourceLength === 0) return bytes(value);
    if (!ArrayBuffer.isView(value)) {
        throw new TypeError("WebGL 2 buffer source ranges require an ArrayBufferView");
    }
    sourceOffset = Number(sourceOffset) >>> 0;
    sourceLength = Number(sourceLength) >>> 0;
    const elementBytes = value.BYTES_PER_ELEMENT ?? 1;
    const elements = value.byteLength / elementBytes;
    if (sourceOffset > elements) throw new RangeError("Buffer source offset is outside the data");
    if (sourceLength === 0) sourceLength = elements - sourceOffset;
    if (sourceLength > elements - sourceOffset) throw new RangeError("Buffer source range is outside the data");
    return new Uint8Array(value.buffer, value.byteOffset + sourceOffset * elementBytes,
        sourceLength * elementBytes);
}

function compressedSourceBytes(context, value, sourceOffset = 0, sourceLength = 0) {
    if (!ArrayBuffer.isView(value)) {
        throw new TypeError("Compressed texture data must be an ArrayBufferView");
    }
    return bufferSourceBytes(context, value, sourceOffset, sourceLength);
}

function uniformValues(value, constructor, multiple, method, sourceOffset = 0, sourceLength = 0) {
    if (value === null || value === undefined || typeof value === "number") {
        throw new TypeError(`${method} expects an array or typed array`);
    }
    let values = value instanceof constructor ? value : new constructor(value);
    sourceOffset = Number(sourceOffset) >>> 0;
    sourceLength = Number(sourceLength) >>> 0;
    if (sourceOffset > values.length) throw new RangeError(`${method} source offset is outside the data`);
    const end = sourceLength === 0 ? values.length : sourceOffset + sourceLength;
    if (end > values.length) throw new RangeError(`${method} source range is outside the data`);
    values = values.subarray(sourceOffset, end);
    if (values.length === 0 || values.length % multiple !== 0) {
        throw new TypeError(`${method} data length must be a non-zero multiple of ${multiple}`);
    }
    return values;
}

function versionedUniformValues(context, value, constructor, multiple, method,
    sourceOffset, sourceLength) {
    return uniformValues(value, constructor, multiple, method,
        context.__version === 2 ? sourceOffset : 0,
        context.__version === 2 ? sourceLength : 0);
}

function textureBytes(value, nullable = false) {
    if (nullable && value === null) return null;
    if (ArrayBuffer.isView(value)) {
        return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
    }
    throw new TypeError("Texture pixels must be null or an ArrayBufferView");
}

function texturePixelBytes(context, format, type) {
    const components = new Map([
        [0x1906, 1], [0x1909, 1], [0x190A, 2],
        [0x1903, 1], [0x8D94, 1], [0x8227, 2], [0x8228, 2],
        [0x1907, 3], [0x8D98, 3], [0x8C40, 3],
        [0x1908, 4], [0x8D99, 4], [0x8C42, 4],
        [0x1902, 1], [0x84F9, 1],
    ]).get(format);
    if (components === undefined) throw new TypeError("Unsupported texture pixel format");
    const packedBytes = new Map([
        [0x8363, 2], [0x8033, 2], [0x8034, 2],
        [0x8368, 4], [0x8C3B, 4], [0x8C3E, 4], [0x84FA, 4], [0x8DAD, 8],
    ]).get(type);
    if (packedBytes !== undefined) return packedBytes;
    const componentBytes = new Map([
        [0x1400, 1], [0x1401, 1], [0x1402, 2], [0x1403, 2],
        [0x140B, 2], [0x8D61, 2], [0x1404, 4], [0x1405, 4], [0x1406, 4],
    ]).get(type);
    if (componentBytes === undefined) throw new TypeError("Unsupported texture pixel type");
    return components * componentBytes;
}

function textureByteLength(context, width, height, format, type) {
    if (width < 0 || height < 0) throw new RangeError("Texture dimensions must be non-negative");
    if (height === 0) return 0;
    const row = width * texturePixelBytes(context, format, type);
    const stride = Math.ceil(row / context.__unpackAlignment) * context.__unpackAlignment;
    return stride * (height - 1) + row;
}

function textureByteLength3D(context, width, height, depth, format, type) {
    if (depth < 0) throw new RangeError("Texture dimensions must be non-negative");
    if (depth === 0 || height === 0) return 0;
    const lastImage = textureByteLength(context, width, height, format, type);
    const row = width * texturePixelBytes(context, format, type);
    const stride = Math.ceil(row / context.__unpackAlignment) * context.__unpackAlignment;
    return stride * height * (depth - 1) + lastImage;
}

function textureSource(source) {
    let result;
    if (typeof globalThis.ImageData === "function" && source instanceof globalThis.ImageData) {
        result = {
            kind: "image-data", width: source.width, height: source.height, payload: source.data,
            originClean: true, colorSpace: source.colorSpace, pixelFormat: source.pixelFormat,
        };
    } else if (source instanceof HTMLCanvasElement) {
        result = {
            kind: "canvas", width: source.width, height: source.height, payload: source,
            originClean: call("canvasOriginClean", source), colorSpace: "native", pixelFormat: "native",
        };
    } else if (source instanceof HTMLImageElement) {
        const metadata = JSON.parse(call("imageMetadata", source));
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
        throw new TypeError("Unsupported TexImageSource");
    }
    if (result.width === 0 || result.height === 0) {
        throw new DOMException("The texture source has no pixels", "InvalidStateError");
    }
    if (!result.originClean) throw new DOMException("The texture source is not origin-clean", "SecurityError");
    return result;
}

function setUniformF(context, location, components, value) {
    if (location === null) return;
    const values = value instanceof Float32Array ? value : new Float32Array(value.map(Number));
    call("webglUniformF", context.canvas, objectId(context, location, WebGLUniformLocation), components, values);
}

function setUniformI(context, location, components, value) {
    if (location === null) return;
    const values = value instanceof Int32Array ? value : new Int32Array(value.map(item => Number(item) | 0));
    call("webglUniformI", context.canvas, objectId(context, location, WebGLUniformLocation), components, values);
}

function setUniformU(context, location, components, value) {
    if (location === null) return;
    const values = value instanceof Uint32Array ? value : new Uint32Array(value.map(item => Number(item) >>> 0));
    call("webglUniformU", context.canvas, objectId(context, location, WebGLUniformLocation), components, values);
}

function setUniformMatrix(context, location, dimension, transpose, value, method,
    sourceOffset = 0, sourceLength = 0) {
    if (location === null) return;
    const values = versionedUniformValues(
        context, value, Float32Array, dimension * dimension, method, sourceOffset, sourceLength);
    call("webglUniformMatrixF", context.canvas, objectId(context, location, WebGLUniformLocation), dimension, Boolean(transpose), values);
}

function setUniformMatrixRect(context, location, columns, rows, transpose, value, method, sourceOffset = 0, sourceLength = 0) {
    if (location === null) return;
    const values = uniformValues(value, Float32Array, columns * rows, method, sourceOffset, sourceLength);
    call("webglUniformMatrixRectF", context.canvas, objectId(context, location, WebGLUniformLocation),
        columns, rows, Boolean(transpose), values);
}

class WebGLRenderingContext {
    constructor(token, canvas, version) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            canvas: { value: canvas, enumerable: true },
            drawingBufferColorSpace: { value: "srgb", writable: true, enumerable: true },
            unpackColorSpace: { value: "srgb", writable: true, enumerable: true },
            __version: { value: version },
            __activeTexture: { value: 0x84C0, writable: true },
            __texture2D: { value: new Map() },
            __textureCube: { value: new Map() },
            __texture3D: { value: new Map() },
            __texture2DArray: { value: new Map() },
            __samplers: { value: new Map() },
            __queries: { value: new Map() },
            __errors: { value: [] },
            __indexedBuffers: { value: new Map() },
            __currentAttributes: { value: new Map() },
            __transformFeedback: { value: null, writable: true },
            __transformFeedbackActive: { value: false, writable: true },
            __transformFeedbackPaused: { value: false, writable: true },
            __vertexArray: { value: null, writable: true },
            __framebuffer: { value: null, writable: true },
            __drawFramebuffer: { value: null, writable: true },
            __readFramebuffer: { value: null, writable: true },
            __renderbuffer: { value: null, writable: true },
            __arrayBuffer: { value: null, writable: true },
            __elementArrayBuffer: { value: null, writable: true },
            __uniformBuffer: { value: null, writable: true },
            __copyReadBuffer: { value: null, writable: true },
            __copyWriteBuffer: { value: null, writable: true },
            __pixelPackBuffer: { value: null, writable: true },
            __pixelUnpackBuffer: { value: null, writable: true },
            __currentProgram: { value: null, writable: true },
            __unpackAlignment: { value: 4, writable: true },
            __unpackFlipY: { value: false, writable: true },
            __unpackPremultiplyAlpha: { value: false, writable: true },
            __objects: { value: new Set() },
            __contextLost: { value: false, writable: true },
            __lostError: { value: false, writable: true },
            __restoreAllowed: { value: false, writable: true },
            __restorePending: { value: false, writable: true },
            __lossExtension: { value: new WebGLLoseContext(construct, this) },
            __extensions: { value: new Map() },
            __compressedTextureFormats: { value: new Set() },
        });
    }
    get drawingBufferWidth() { return this.canvas.width; }
    get drawingBufferHeight() { return this.canvas.height; }
    clearColor(red, green, blue, alpha) {
        call("webglClearColor", this.canvas, Number(red), Number(green), Number(blue), Number(alpha));
    }
    clear(mask) { call("webglClear", this.canvas, Number(mask) >>> 0); }
    clearDepth(depth) { call("webglClearDepth", this.canvas, Number(depth)); }
    clearStencil(stencil) { call("webglClearStencil", this.canvas, Number(stencil) | 0); }
    enable(capability) { call("webglSetEnabled", this.canvas, Number(capability) >>> 0, true); }
    disable(capability) { call("webglSetEnabled", this.canvas, Number(capability) >>> 0, false); }
    isEnabled(capability) {
        return this.__contextLost ? false : call("webglIsEnabled", this.canvas, Number(capability) >>> 0);
    }
    scissor(x, y, width, height) {
        call("webglScissor", this.canvas, Number(x) | 0, Number(y) | 0, Number(width) | 0, Number(height) | 0);
    }
    colorMask(red, green, blue, alpha) {
        call("webglColorMask", this.canvas, Boolean(red), Boolean(green), Boolean(blue), Boolean(alpha));
    }
    depthMask(value) { call("webglDepthMask", this.canvas, Boolean(value)); }
    depthFunc(value) { call("webglDepthFunc", this.canvas, Number(value) >>> 0); }
    depthRange(near, far) { call("webglDepthRange", this.canvas, Number(near), Number(far)); }
    blendFunc(source, destination) {
        call("webglBlendFunc", this.canvas, Number(source) >>> 0, Number(destination) >>> 0);
    }
    blendColor(red, green, blue, alpha) {
        call("webglBlendColor", this.canvas, Number(red), Number(green), Number(blue), Number(alpha));
    }
    blendFuncSeparate(sourceRGB, destinationRGB, sourceAlpha, destinationAlpha) {
        call("webglBlendFuncSeparate", this.canvas, Number(sourceRGB) >>> 0, Number(destinationRGB) >>> 0, Number(sourceAlpha) >>> 0, Number(destinationAlpha) >>> 0);
    }
    blendEquation(mode) { call("webglBlendEquation", this.canvas, Number(mode) >>> 0); }
    blendEquationSeparate(modeRGB, modeAlpha) {
        call("webglBlendEquationSeparate", this.canvas, Number(modeRGB) >>> 0, Number(modeAlpha) >>> 0);
    }
    stencilFunc(func, reference, mask) {
        call("webglStencilFunc", this.canvas, Number(func) >>> 0, Number(reference) | 0, Number(mask) >>> 0);
    }
    stencilFuncSeparate(face, func, reference, mask) {
        call("webglStencilFuncSeparate", this.canvas, Number(face) >>> 0, Number(func) >>> 0, Number(reference) | 0, Number(mask) >>> 0);
    }
    stencilMask(mask) { call("webglStencilMask", this.canvas, Number(mask) >>> 0); }
    stencilMaskSeparate(face, mask) {
        call("webglStencilMaskSeparate", this.canvas, Number(face) >>> 0, Number(mask) >>> 0);
    }
    stencilOp(fail, depthFail, pass) {
        call("webglStencilOp", this.canvas, Number(fail) >>> 0, Number(depthFail) >>> 0, Number(pass) >>> 0);
    }
    stencilOpSeparate(face, fail, depthFail, pass) {
        call("webglStencilOpSeparate", this.canvas, Number(face) >>> 0, Number(fail) >>> 0, Number(depthFail) >>> 0, Number(pass) >>> 0);
    }
    polygonOffset(factor, units) {
        call("webglPolygonOffset", this.canvas, Number(factor), Number(units));
    }
    sampleCoverage(value, invert) {
        call("webglSampleCoverage", this.canvas, Number(value), Boolean(invert));
    }
    cullFace(face) { call("webglCullFace", this.canvas, Number(face) >>> 0); }
    frontFace(winding) { call("webglFrontFace", this.canvas, Number(winding) >>> 0); }
    lineWidth(width) { call("webglLineWidth", this.canvas, Number(width)); }
    flush() { call("webglFlush", this.canvas); }
    finish() { call("webglFinish", this.canvas); }
    hint(target, mode) {
        call("webglHint", this.canvas, Number(target) >>> 0, Number(mode) >>> 0);
    }
    createShader(type) {
        if (this.__contextLost) return null;
        type = Number(type) >>> 0;
        return new WebGLShader(construct, this, call("webglCreateShader", this.canvas, type), type);
    }
    shaderSource(shader, source) {
        source = String(source);
        call("webglShaderSource", this.canvas, objectId(this, shader, WebGLShader), source);
        shader.__source = source;
    }
    compileShader(shader) {
        call("webglCompileShader", this.canvas, objectId(this, shader, WebGLShader));
    }
    getShaderParameter(shader, parameter) {
        if (this.__contextLost) return null;
        const id = objectId(this, shader, WebGLShader);
        parameter = Number(parameter) >>> 0;
        if (parameter === this.COMPILE_STATUS) return call("webglShaderStatus", this.canvas, id);
        if (parameter === this.SHADER_TYPE) return shader.__type;
        if (parameter === this.DELETE_STATUS) return false;
        return null;
    }
    getShaderInfoLog(shader) {
        if (this.__contextLost) return null;
        return call("webglShaderLog", this.canvas, objectId(this, shader, WebGLShader));
    }
    getShaderSource(shader) {
        objectId(this, shader, WebGLShader);
        return shader.__source;
    }
    getShaderPrecisionFormat(shaderType, precisionType) {
        const encoded = call("webglShaderPrecisionFormat", this.canvas,
            Number(shaderType) >>> 0, Number(precisionType) >>> 0);
        if (encoded === null) return null;
        const value = JSON.parse(encoded);
        return new WebGLShaderPrecisionFormat(
            construct, value.rangeMin, value.rangeMax, value.precision);
    }
    deleteShader(shader) {
        if (shader === null || shader?.__id === 0) return;
        call("webglDeleteShader", this.canvas, objectId(this, shader, WebGLShader));
        shader.__id = 0;
    }
    isShader(shader) { return shader instanceof WebGLShader && shader.__context === this && shader.__id !== 0; }
    createProgram() {
        if (this.__contextLost) return null;
        return new WebGLProgram(construct, this, call("webglCreateProgram", this.canvas));
    }
    attachShader(program, shader) {
        call("webglAttachShader", this.canvas, objectId(this, program, WebGLProgram), objectId(this, shader, WebGLShader));
        program.__attachedShaders.add(shader);
    }
    detachShader(program, shader) {
        call("webglDetachShader", this.canvas, objectId(this, program, WebGLProgram), objectId(this, shader, WebGLShader));
        program.__attachedShaders.delete(shader);
    }
    bindAttribLocation(program, index, name) {
        call("webglBindAttribLocation", this.canvas, objectId(this, program, WebGLProgram), Number(index) >>> 0, String(name));
    }
    linkProgram(program) {
        call("webglLinkProgram", this.canvas, objectId(this, program, WebGLProgram));
    }
    validateProgram(program) {
        call("webglValidateProgram", this.canvas, objectId(this, program, WebGLProgram));
    }
    getProgramParameter(program, parameter) {
        parameter = Number(parameter) >>> 0;
        const parallelCompilation = this.__extensions.has("KHR_parallel_shader_compile")
            && parameter === 0x91B1;
        if (this.__contextLost) return parallelCompilation ? true : null;
        const id = objectId(this, program, WebGLProgram);
        if (parallelCompilation) return Boolean(call("webglProgramParameter", this.canvas, id, parameter));
        if (parameter === this.LINK_STATUS) return call("webglProgramStatus", this.canvas, id);
        if (parameter === this.VALIDATE_STATUS) return call("webglProgramValidateStatus", this.canvas, id);
        if (parameter === this.DELETE_STATUS) return false;
        if (parameter === this.ATTACHED_SHADERS) return program.__attachedShaders.size;
        if ([this.ACTIVE_ATTRIBUTES, this.ACTIVE_UNIFORMS, this.ACTIVE_UNIFORM_BLOCKS,
            this.TRANSFORM_FEEDBACK_VARYINGS, this.TRANSFORM_FEEDBACK_BUFFER_MODE].includes(parameter)) {
            return call("webglProgramParameter", this.canvas, id, parameter);
        }
        return null;
    }
    getActiveAttrib(program, index) {
        const encoded = call("webglGetActiveAttrib", this.canvas,
            objectId(this, program, WebGLProgram), Number(index) >>> 0);
        if (encoded === null) return null;
        const value = JSON.parse(encoded);
        return new WebGLActiveInfo(construct, value.name, value.size, value.type);
    }
    getActiveUniform(program, index) {
        const encoded = call("webglGetActiveUniform", this.canvas,
            objectId(this, program, WebGLProgram), Number(index) >>> 0);
        if (encoded === null) return null;
        const value = JSON.parse(encoded);
        return new WebGLActiveInfo(construct, value.name, value.size, value.type);
    }
    getAttachedShaders(program) {
        objectId(this, program, WebGLProgram);
        return [...program.__attachedShaders];
    }
    getProgramInfoLog(program) {
        if (this.__contextLost) return null;
        return call("webglProgramLog", this.canvas, objectId(this, program, WebGLProgram));
    }
    useProgram(program) {
        call("webglUseProgram", this.canvas, objectId(this, program, WebGLProgram, true));
        this.__currentProgram = program;
    }
    deleteProgram(program) {
        if (program === null || program?.__id === 0) return;
        call("webglDeleteProgram", this.canvas, objectId(this, program, WebGLProgram));
        program.__id = 0;
    }
    isProgram(program) { return program instanceof WebGLProgram && program.__context === this && program.__id !== 0; }
    createBuffer() {
        if (this.__contextLost) return null;
        return new WebGLBuffer(construct, this, call("webglCreateBuffer", this.canvas));
    }
    bindBuffer(target, buffer) {
        target = Number(target) >>> 0;
        call("webglBindBuffer", this.canvas, target, objectId(this, buffer, WebGLBuffer, true));
        if (target === this.ARRAY_BUFFER) this.__arrayBuffer = buffer;
        if (target === this.ELEMENT_ARRAY_BUFFER) this.__elementArrayBuffer = buffer;
        if (target === this.UNIFORM_BUFFER) this.__uniformBuffer = buffer;
        if (target === this.COPY_READ_BUFFER) this.__copyReadBuffer = buffer;
        if (target === this.COPY_WRITE_BUFFER) this.__copyWriteBuffer = buffer;
        if (target === this.PIXEL_PACK_BUFFER) this.__pixelPackBuffer = buffer;
        if (target === this.PIXEL_UNPACK_BUFFER) this.__pixelUnpackBuffer = buffer;
    }
    bufferData(target, data, usage, sourceOffset = 0, sourceLength = 0) {
        call("webglBufferData", this.canvas, Number(target) >>> 0,
            bufferSourceBytes(this, data, sourceOffset, sourceLength), Number(usage) >>> 0);
    }
    bufferSubData(target, offset, data, sourceOffset = 0, sourceLength = 0) {
        call("webglBufferSubData", this.canvas, Number(target) >>> 0, Number(offset) | 0,
            bufferSourceBytes(this, data, sourceOffset, sourceLength));
    }
    getBufferParameter(target, parameter) {
        return call("webglGetBufferParameter", this.canvas,
            Number(target) >>> 0, Number(parameter) >>> 0);
    }
    deleteBuffer(buffer) {
        if (buffer === null || buffer?.__id === 0) return;
        call("webglDeleteBuffer", this.canvas, objectId(this, buffer, WebGLBuffer));
        if (this.__arrayBuffer === buffer) this.__arrayBuffer = null;
        if (this.__elementArrayBuffer === buffer) this.__elementArrayBuffer = null;
        if (this.__uniformBuffer === buffer) this.__uniformBuffer = null;
        if (this.__copyReadBuffer === buffer) this.__copyReadBuffer = null;
        if (this.__copyWriteBuffer === buffer) this.__copyWriteBuffer = null;
        if (this.__pixelPackBuffer === buffer) this.__pixelPackBuffer = null;
        if (this.__pixelUnpackBuffer === buffer) this.__pixelUnpackBuffer = null;
        for (const [key, binding] of this.__indexedBuffers) {
            if (binding.buffer === buffer) this.__indexedBuffers.set(key, { buffer: null, offset: 0, size: 0 });
        }
        buffer.__id = 0;
    }
    isBuffer(buffer) { return buffer instanceof WebGLBuffer && buffer.__context === this && buffer.__id !== 0; }
    createTexture() {
        if (this.__contextLost) return null;
        return new WebGLTexture(construct, this, call("webglCreateTexture", this.canvas));
    }
    activeTexture(texture) {
        texture = Number(texture) >>> 0;
        call("webglActiveTexture", this.canvas, texture);
        this.__activeTexture = texture;
    }
    bindTexture(target, texture) {
        target = Number(target) >>> 0;
        const id = objectId(this, texture, WebGLTexture, true);
        call("webglBindTexture", this.canvas, target, id);
        if (target === this.TEXTURE_2D) this.__texture2D.set(this.__activeTexture, texture);
        if (target === this.TEXTURE_CUBE_MAP) this.__textureCube.set(this.__activeTexture, texture);
        if (target === this.TEXTURE_3D) this.__texture3D.set(this.__activeTexture, texture);
        if (target === this.TEXTURE_2D_ARRAY) this.__texture2DArray.set(this.__activeTexture, texture);
    }
    texImage2D(target, level, internalFormat, ...arguments_) {
        target = Number(target) >>> 0;
        level = Number(level) | 0;
        internalFormat = Number(internalFormat) | 0;
        if (arguments_.length === 6 || arguments_.length === 7) {
            let [width, height, border, format, type, pixels, sourceOffset = 0] = arguments_;
            width = Number(width) | 0;
            height = Number(height) | 0;
            border = Number(border) | 0;
            format = Number(format) >>> 0;
            type = Number(type) >>> 0;
            if (typeof pixels === "number") {
                if (this.__version !== 2 || this.__pixelUnpackBuffer === null) {
                    throw new TypeError("A pixel unpack buffer must be bound for offset upload");
                }
                const offset = Number(pixels) | 0;
                if (offset < 0) throw new RangeError("Pixel unpack buffer offset must be non-negative");
                call("webglTexImage2DOffset", this.canvas, target, level, internalFormat,
                    width, height, border, format, type, offset);
                return;
            }
            const rangedPixels = arguments_.length === 7
                ? bufferSourceBytes(this, pixels, sourceOffset, 0) : pixels;
            const data = textureBytes(rangedPixels, true);
            const required = textureByteLength(this, width, height, format, type);
            if (data !== null && data.byteLength < required) throw new RangeError("Texture pixel data is too small");
            call("webglTexImage2D", this.canvas, target, level, internalFormat, width, height, border, format, type, data !== null, data ?? new Uint8Array());
            return;
        }
        if (arguments_.length !== 3) throw new TypeError("texImage2D requires 6 or 9 arguments");
        let [format, type, source] = arguments_;
        format = Number(format) >>> 0;
        type = Number(type) >>> 0;
        if (format !== this.RGBA || type !== this.UNSIGNED_BYTE) {
            throw new TypeError("CanvasImageSource uploads currently require RGBA/UNSIGNED_BYTE");
        }
        const { kind, width, height, payload, originClean, colorSpace, pixelFormat } = textureSource(source);
        call("webglTexImageSource", this.canvas, target, level, internalFormat, format, type,
            kind, width, height, payload, this.__unpackFlipY, this.__unpackPremultiplyAlpha, originClean,
            colorSpace, pixelFormat);
    }
    texSubImage2D(target, level, xoffset, yoffset, ...arguments_) {
        target = Number(target) >>> 0;
        level = Number(level) | 0;
        xoffset = Number(xoffset) | 0;
        yoffset = Number(yoffset) | 0;
        if (arguments_.length === 5 || arguments_.length === 6) {
            let [width, height, format, type, pixels, sourceOffset = 0] = arguments_;
            width = Number(width) | 0;
            height = Number(height) | 0;
            format = Number(format) >>> 0;
            type = Number(type) >>> 0;
            if (typeof pixels === "number") {
                if (this.__version !== 2 || this.__pixelUnpackBuffer === null) {
                    throw new TypeError("A pixel unpack buffer must be bound for offset upload");
                }
                const offset = Number(pixels) | 0;
                if (offset < 0) throw new RangeError("Pixel unpack buffer offset must be non-negative");
                call("webglTexSubImage2DOffset", this.canvas, target, level, xoffset, yoffset,
                    width, height, format, type, offset);
                return;
            }
            const rangedPixels = arguments_.length === 6
                ? bufferSourceBytes(this, pixels, sourceOffset, 0) : pixels;
            const data = textureBytes(rangedPixels);
            if (data.byteLength < textureByteLength(this, width, height, format, type)) {
                throw new RangeError("Texture pixel data is too small");
            }
            call("webglTexSubImage2D", this.canvas, target, level, xoffset, yoffset, width, height, format, type, data);
            return;
        }
        if (arguments_.length !== 3) throw new TypeError("texSubImage2D requires 7 or 9 arguments");
        let [format, type, source] = arguments_;
        format = Number(format) >>> 0;
        type = Number(type) >>> 0;
        if (format !== this.RGBA || type !== this.UNSIGNED_BYTE) {
            throw new TypeError("CanvasImageSource uploads currently require RGBA/UNSIGNED_BYTE");
        }
        const { kind, width, height, payload, originClean, colorSpace, pixelFormat } = textureSource(source);
        call("webglTexSubImageSource", this.canvas, target, level, xoffset, yoffset, format, type,
            kind, width, height, payload, this.__unpackFlipY, this.__unpackPremultiplyAlpha, originClean,
            colorSpace, pixelFormat);
    }
    copyTexImage2D(target, level, internalFormat, x, y, width, height, border) {
        call(
            "webglCopyTexImage2D",
            this.canvas,
            Number(target) >>> 0,
            Number(level) | 0,
            Number(internalFormat) >>> 0,
            Number(x) | 0,
            Number(y) | 0,
            Number(width) | 0,
            Number(height) | 0,
            Number(border) | 0,
        );
    }
    copyTexSubImage2D(target, level, xoffset, yoffset, x, y, width, height) {
        call(
            "webglCopyTexSubImage2D",
            this.canvas,
            Number(target) >>> 0,
            Number(level) | 0,
            Number(xoffset) | 0,
            Number(yoffset) | 0,
            Number(x) | 0,
            Number(y) | 0,
            Number(width) | 0,
            Number(height) | 0,
        );
    }
    compressedTexImage2D(target, level, internalFormat, width, height, border, dataOrSize,
        sourceOffset = 0, sourceLengthOverride = 0) {
        target = Number(target) >>> 0;
        level = Number(level) | 0;
        internalFormat = Number(internalFormat) >>> 0;
        width = Number(width) | 0;
        height = Number(height) | 0;
        border = Number(border) | 0;
        if (typeof dataOrSize === "number") {
            if (this.__version !== 2 || this.__pixelUnpackBuffer === null) {
                throw new TypeError("A pixel unpack buffer must be bound for compressed offset upload");
            }
            call("webglCompressedTexImage2DOffset", this.canvas, target, level, internalFormat,
                width, height, border, Number(dataOrSize) | 0, Number(sourceOffset) >>> 0);
            return;
        }
        const data = compressedSourceBytes(
            this, dataOrSize, sourceOffset, sourceLengthOverride);
        call("webglCompressedTexImage2D", this.canvas, target, level, internalFormat,
            width, height, border, data);
    }
    compressedTexSubImage2D(target, level, xoffset, yoffset, width, height, format, dataOrSize,
        sourceOffset = 0, sourceLengthOverride = 0) {
        target = Number(target) >>> 0;
        level = Number(level) | 0;
        xoffset = Number(xoffset) | 0;
        yoffset = Number(yoffset) | 0;
        width = Number(width) | 0;
        height = Number(height) | 0;
        format = Number(format) >>> 0;
        if (!this.__contextLost
            && format === 0x8D64
            && this.__compressedTextureFormats.has(format)) {
            this.__errors.push(this.INVALID_OPERATION);
            return;
        }
        if (typeof dataOrSize === "number") {
            if (this.__version !== 2 || this.__pixelUnpackBuffer === null) {
                throw new TypeError("A pixel unpack buffer must be bound for compressed offset upload");
            }
            call("webglCompressedTexSubImage2DOffset", this.canvas, target, level,
                xoffset, yoffset, width, height, format, Number(dataOrSize) >>> 0,
                Number(sourceOffset) >>> 0);
            return;
        }
        const data = compressedSourceBytes(
            this, dataOrSize, sourceOffset, sourceLengthOverride);
        call("webglCompressedTexSubImage2D", this.canvas, target, level,
            xoffset, yoffset, width, height, format, data);
    }
    texParameteri(target, parameter, value) {
        call("webglTexParameteri", this.canvas, Number(target) >>> 0, Number(parameter) >>> 0, Number(value) | 0);
    }
    texParameterf(target, parameter, value) {
        call("webglTexParameterf", this.canvas, Number(target) >>> 0,
            Number(parameter) >>> 0, Number(value));
    }
    getTexParameter(target, parameter) {
        target = Number(target) >>> 0;
        parameter = Number(parameter) >>> 0;
        const value = call(parameter === 0x813A || parameter === 0x813B || parameter === 0x84FE
            ? "webglGetTexParameterF" : "webglGetTexParameterI",
            this.canvas, target, parameter);
        return parameter === this.TEXTURE_IMMUTABLE_FORMAT ? Boolean(value) : value;
    }
    pixelStorei(parameter, value) {
        parameter = Number(parameter) >>> 0;
        value = Number(value) | 0;
        if (parameter === this.UNPACK_FLIP_Y_WEBGL) { this.__unpackFlipY = Boolean(value); return; }
        if (parameter === this.UNPACK_PREMULTIPLY_ALPHA_WEBGL) { this.__unpackPremultiplyAlpha = Boolean(value); return; }
        if (parameter === this.UNPACK_COLORSPACE_CONVERSION_WEBGL) return;
        if (parameter === this.UNPACK_ALIGNMENT) {
            if (![1, 2, 4, 8].includes(value)) throw new RangeError("UNPACK_ALIGNMENT must be 1, 2, 4, or 8");
            this.__unpackAlignment = value;
        }
        call("webglPixelStorei", this.canvas, parameter, value);
    }
    generateMipmap(target) { call("webglGenerateMipmap", this.canvas, Number(target) >>> 0); }
    deleteTexture(texture) {
        if (texture === null || texture?.__id === 0) return;
        call("webglDeleteTexture", this.canvas, objectId(this, texture, WebGLTexture));
        for (const [unit, binding] of this.__texture2D) {
            if (binding === texture) this.__texture2D.set(unit, null);
        }
        for (const bindings of [this.__textureCube, this.__texture3D, this.__texture2DArray]) {
            for (const [unit, binding] of bindings) {
                if (binding === texture) bindings.set(unit, null);
            }
        }
        texture.__id = 0;
    }
    isTexture(texture) { return texture instanceof WebGLTexture && texture.__context === this && texture.__id !== 0; }
    createFramebuffer() {
        if (this.__contextLost) return null;
        return new WebGLFramebuffer(construct, this, call("webglCreateFramebuffer", this.canvas));
    }
    bindFramebuffer(target, framebuffer) {
        target = Number(target) >>> 0;
        call("webglBindFramebuffer", this.canvas, target, objectId(this, framebuffer, WebGLFramebuffer, true));
        if (target === this.FRAMEBUFFER) {
            this.__framebuffer = framebuffer;
            this.__drawFramebuffer = framebuffer;
            this.__readFramebuffer = framebuffer;
        }
        if (target === this.DRAW_FRAMEBUFFER) {
            this.__framebuffer = framebuffer;
            this.__drawFramebuffer = framebuffer;
        }
        if (target === this.READ_FRAMEBUFFER) this.__readFramebuffer = framebuffer;
    }
    framebufferTexture2D(target, attachment, textureTarget, texture, level) {
        call("webglFramebufferTexture2D", this.canvas, Number(target) >>> 0, Number(attachment) >>> 0, Number(textureTarget) >>> 0, objectId(this, texture, WebGLTexture, true), Number(level) | 0);
    }
    getFramebufferAttachmentParameter(target, attachment, parameter) {
        target = Number(target) >>> 0;
        attachment = Number(attachment) >>> 0;
        parameter = Number(parameter) >>> 0;
        const value = call("webglGetFramebufferAttachmentParameter", this.canvas,
            target, attachment, parameter);
        if (parameter !== this.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME) return value;
        const object = JSON.parse(value);
        if (object.id === null) return null;
        if (object.type === this.TEXTURE) return objectById(this, object.id, WebGLTexture);
        if (object.type === this.RENDERBUFFER) return objectById(this, object.id, WebGLRenderbuffer);
        return null;
    }
    checkFramebufferStatus(target) {
        if (this.__contextLost) return this.FRAMEBUFFER_UNSUPPORTED;
        return call("webglCheckFramebufferStatus", this.canvas, Number(target) >>> 0);
    }
    deleteFramebuffer(framebuffer) {
        if (framebuffer === null || framebuffer?.__id === 0) return;
        call("webglDeleteFramebuffer", this.canvas, objectId(this, framebuffer, WebGLFramebuffer));
        if (this.__framebuffer === framebuffer) this.__framebuffer = null;
        if (this.__drawFramebuffer === framebuffer) this.__drawFramebuffer = null;
        if (this.__readFramebuffer === framebuffer) this.__readFramebuffer = null;
        framebuffer.__id = 0;
    }
    isFramebuffer(framebuffer) { return framebuffer instanceof WebGLFramebuffer && framebuffer.__context === this && framebuffer.__id !== 0; }
    createRenderbuffer() {
        if (this.__contextLost) return null;
        return new WebGLRenderbuffer(construct, this, call("webglCreateRenderbuffer", this.canvas));
    }
    bindRenderbuffer(target, renderbuffer) {
        call("webglBindRenderbuffer", this.canvas, Number(target) >>> 0, objectId(this, renderbuffer, WebGLRenderbuffer, true));
        if ((Number(target) >>> 0) === this.RENDERBUFFER) this.__renderbuffer = renderbuffer;
    }
    renderbufferStorage(target, internalFormat, width, height) {
        call("webglRenderbufferStorage", this.canvas, Number(target) >>> 0, Number(internalFormat) >>> 0, Number(width) | 0, Number(height) | 0);
    }
    getRenderbufferParameter(target, parameter) {
        return call("webglGetRenderbufferParameter", this.canvas,
            Number(target) >>> 0, Number(parameter) >>> 0);
    }
    framebufferRenderbuffer(target, attachment, renderbufferTarget, renderbuffer) {
        call("webglFramebufferRenderbuffer", this.canvas, Number(target) >>> 0, Number(attachment) >>> 0, Number(renderbufferTarget) >>> 0, objectId(this, renderbuffer, WebGLRenderbuffer, true));
    }
    deleteRenderbuffer(renderbuffer) {
        if (renderbuffer === null || renderbuffer?.__id === 0) return;
        call("webglDeleteRenderbuffer", this.canvas, objectId(this, renderbuffer, WebGLRenderbuffer));
        if (this.__renderbuffer === renderbuffer) this.__renderbuffer = null;
        renderbuffer.__id = 0;
    }
    isRenderbuffer(renderbuffer) { return renderbuffer instanceof WebGLRenderbuffer && renderbuffer.__context === this && renderbuffer.__id !== 0; }
    getAttribLocation(program, name) {
        if (this.__contextLost) return -1;
        return call("webglGetAttribLocation", this.canvas, objectId(this, program, WebGLProgram), String(name));
    }
    enableVertexAttribArray(index) {
        call("webglEnableVertexAttribArray", this.canvas, Number(index) >>> 0);
    }
    disableVertexAttribArray(index) {
        call("webglDisableVertexAttribArray", this.canvas, Number(index) >>> 0);
    }
    vertexAttribPointer(index, size, type, normalized, stride, offset) {
        call("webglVertexAttribPointer", this.canvas, Number(index) >>> 0, Number(size), Number(type) >>> 0, Boolean(normalized), Number(stride), Number(offset));
    }
    vertexAttrib1f(index, x) { this.vertexAttrib4f(index, x, 0, 0, 1); }
    vertexAttrib2f(index, x, y) { this.vertexAttrib4f(index, x, y, 0, 1); }
    vertexAttrib3f(index, x, y, z) { this.vertexAttrib4f(index, x, y, z, 1); }
    vertexAttrib4f(index, x, y, z, w) {
        index = Number(index) >>> 0;
        const values = new Float32Array([x, y, z, w]);
        call("webglVertexAttribF", this.canvas, index, values);
        this.__currentAttributes.set(index, values);
    }
    vertexAttrib1fv(index, values) {
        const data = values instanceof Float32Array ? values : new Float32Array(values);
        if (data.length < 1) throw new RangeError("vertexAttrib1fv requires one value");
        this.vertexAttrib1f(index, data[0]);
    }
    vertexAttrib2fv(index, values) {
        const data = values instanceof Float32Array ? values : new Float32Array(values);
        if (data.length < 2) throw new RangeError("vertexAttrib2fv requires two values");
        this.vertexAttrib2f(index, data[0], data[1]);
    }
    vertexAttrib3fv(index, values) {
        const data = values instanceof Float32Array ? values : new Float32Array(values);
        if (data.length < 3) throw new RangeError("vertexAttrib3fv requires three values");
        this.vertexAttrib3f(index, data[0], data[1], data[2]);
    }
    vertexAttrib4fv(index, values) {
        const data = values instanceof Float32Array ? values : new Float32Array(values);
        if (data.length < 4) throw new RangeError("vertexAttrib4fv requires four values");
        this.vertexAttrib4f(index, data[0], data[1], data[2], data[3]);
    }
    getVertexAttrib(index, parameter) {
        index = Number(index) >>> 0;
        parameter = Number(parameter) >>> 0;
        if (parameter === this.CURRENT_VERTEX_ATTRIB) {
            return this.__currentAttributes.get(index)
                ?? new Float32Array(JSON.parse(call("webglGetVertexAttribCurrent", this.canvas, index)));
        }
        if (parameter === this.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING) {
            const id = call("webglGetVertexAttribBuffer", this.canvas, index);
            return objectById(this, id, WebGLBuffer);
        }
        const value = call("webglGetVertexAttribI", this.canvas, index, parameter);
        if ([this.VERTEX_ATTRIB_ARRAY_ENABLED, this.VERTEX_ATTRIB_ARRAY_NORMALIZED,
            this.VERTEX_ATTRIB_ARRAY_INTEGER].includes(parameter)) return Boolean(value);
        return value;
    }
    getVertexAttribOffset(index, parameter) {
        return call("webglGetVertexAttribOffset", this.canvas,
            Number(index) >>> 0, Number(parameter) >>> 0);
    }
    getUniformLocation(program, name) {
        if (this.__contextLost) return null;
        name = String(name);
        const id = call("webglGetUniformLocation", this.canvas, objectId(this, program, WebGLProgram), name);
        return id === null ? null : new WebGLUniformLocation(construct, this, id, name);
    }
    getUniform(program, location) {
        const programId = objectId(this, program, WebGLProgram);
        const locationId = objectId(this, location, WebGLUniformLocation);
        const requestedName = location.__name;
        const baseName = requestedName.replace(/\[\d+\]$/, "[0]");
        const rootName = requestedName.replace(/\[\d+\]$/, "");
        let type = null;
        const count = this.getProgramParameter(program, this.ACTIVE_UNIFORMS);
        for (let index = 0; index < count; index += 1) {
            const info = this.getActiveUniform(program, index);
            if (info !== null && (info.name === requestedName || info.name === baseName
                || info.name.replace(/\[0\]$/, "") === rootName)) {
                type = info.type;
                break;
            }
        }
        if (type === null) return null;
        const encoded = JSON.parse(call("webglGetUniform", this.canvas, programId, locationId, type));
        const scalar = encoded.values.length === 1;
        if (type === this.BOOL && scalar) return Boolean(encoded.values[0]);
        if (scalar) return encoded.values[0];
        if (encoded.kind === "float") return new Float32Array(encoded.values);
        if (encoded.kind === "uint") return new Uint32Array(encoded.values);
        return new Int32Array(encoded.values);
    }
    uniform1f(location, x) { setUniformF(this, location, 1, [x]); }
    uniform2f(location, x, y) { setUniformF(this, location, 2, [x, y]); }
    uniform3f(location, x, y, z) { setUniformF(this, location, 3, [x, y, z]); }
    uniform4f(location, x, y, z, w) {
        setUniformF(this, location, 4, [x, y, z, w]);
    }
    uniform1fv(location, value, sourceOffset = 0, sourceLength = 0) { setUniformF(this, location, 1, versionedUniformValues(this, value, Float32Array, 1, "uniform1fv", sourceOffset, sourceLength)); }
    uniform2fv(location, value, sourceOffset = 0, sourceLength = 0) { setUniformF(this, location, 2, versionedUniformValues(this, value, Float32Array, 2, "uniform2fv", sourceOffset, sourceLength)); }
    uniform3fv(location, value, sourceOffset = 0, sourceLength = 0) { setUniformF(this, location, 3, versionedUniformValues(this, value, Float32Array, 3, "uniform3fv", sourceOffset, sourceLength)); }
    uniform4fv(location, value, sourceOffset = 0, sourceLength = 0) { setUniformF(this, location, 4, versionedUniformValues(this, value, Float32Array, 4, "uniform4fv", sourceOffset, sourceLength)); }
    uniform1i(location, value) {
        setUniformI(this, location, 1, [value]);
    }
    uniform2i(location, x, y) { setUniformI(this, location, 2, [x, y]); }
    uniform3i(location, x, y, z) { setUniformI(this, location, 3, [x, y, z]); }
    uniform4i(location, x, y, z, w) { setUniformI(this, location, 4, [x, y, z, w]); }
    uniform1iv(location, value, sourceOffset = 0, sourceLength = 0) { setUniformI(this, location, 1, versionedUniformValues(this, value, Int32Array, 1, "uniform1iv", sourceOffset, sourceLength)); }
    uniform2iv(location, value, sourceOffset = 0, sourceLength = 0) { setUniformI(this, location, 2, versionedUniformValues(this, value, Int32Array, 2, "uniform2iv", sourceOffset, sourceLength)); }
    uniform3iv(location, value, sourceOffset = 0, sourceLength = 0) { setUniformI(this, location, 3, versionedUniformValues(this, value, Int32Array, 3, "uniform3iv", sourceOffset, sourceLength)); }
    uniform4iv(location, value, sourceOffset = 0, sourceLength = 0) { setUniformI(this, location, 4, versionedUniformValues(this, value, Int32Array, 4, "uniform4iv", sourceOffset, sourceLength)); }
    uniformMatrix2fv(location, transpose, value, sourceOffset = 0, sourceLength = 0) { setUniformMatrix(this, location, 2, transpose, value, "uniformMatrix2fv", sourceOffset, sourceLength); }
    uniformMatrix3fv(location, transpose, value, sourceOffset = 0, sourceLength = 0) { setUniformMatrix(this, location, 3, transpose, value, "uniformMatrix3fv", sourceOffset, sourceLength); }
    uniformMatrix4fv(location, transpose, value, sourceOffset = 0, sourceLength = 0) { setUniformMatrix(this, location, 4, transpose, value, "uniformMatrix4fv", sourceOffset, sourceLength); }
    viewport(x, y, width, height) {
        call("webglViewport", this.canvas, Number(x), Number(y), Number(width), Number(height));
    }
    drawArrays(mode, first, count) {
        call("webglDrawArrays", this.canvas, Number(mode) >>> 0, Number(first), Number(count));
    }
    drawElements(mode, count, type, offset) {
        call("webglDrawElements", this.canvas, Number(mode) >>> 0, Number(count) | 0, Number(type) >>> 0, Number(offset) | 0);
    }
    getParameter(parameter) {
        if (this.__contextLost) return null;
        parameter = Number(parameter) >>> 0;
        if (parameter === 0x88FC
            && !this.__extensions.has("WEBGL_blend_func_extended")) {
            this.__errors.push(this.INVALID_ENUM);
            return null;
        }
        if ((parameter === 0x0B40 || parameter === 0x2A02)
            && !this.__extensions.has("WEBGL_polygon_mode")) {
            this.__errors.push(this.INVALID_ENUM);
            return null;
        }
        if ([this.VENDOR, this.RENDERER, this.VERSION, this.SHADING_LANGUAGE_VERSION].includes(parameter)) {
            if (graphicsPersona !== null) {
                if (parameter === this.VENDOR) return graphicsPersona.webgl_masked_vendor;
                if (parameter === this.RENDERER) return graphicsPersona.webgl_masked_renderer;
            }
            return call("webglGetString", this.canvas, parameter);
        }
        if (parameter === this.UNMASKED_VENDOR_WEBGL || parameter === this.UNMASKED_RENDERER_WEBGL) {
            if (graphicsPersona !== null) {
                return parameter === this.UNMASKED_VENDOR_WEBGL
                    ? graphicsPersona.webgl_vendor
                    : graphicsPersona.webgl_renderer;
            }
            return call("webglGetString", this.canvas, parameter === this.UNMASKED_VENDOR_WEBGL ? this.VENDOR : this.RENDERER);
        }
        if (parameter === this.ACTIVE_TEXTURE) return this.__activeTexture;
        if (parameter === this.COMPRESSED_TEXTURE_FORMATS) {
            return new Uint32Array(this.__compressedTextureFormats);
        }
        if (parameter === 0x8FBB) return call("webglGetBoolean", this.canvas, parameter);
        if (parameter >= 0x3000 && parameter <= 0x3007) {
            return call("webglGetBoolean", this.canvas, parameter);
        }
        if ([0x8E28, 0x8D6B, 0x9111].includes(parameter)) {
            return call("webglGetInteger64", this.canvas, parameter);
        }
        if (parameter === this.TEXTURE_BINDING_2D) return this.__texture2D.get(this.__activeTexture) ?? null;
        if (parameter === this.TEXTURE_BINDING_CUBE_MAP) return this.__textureCube.get(this.__activeTexture) ?? null;
        if (parameter === this.TEXTURE_BINDING_3D) return this.__texture3D.get(this.__activeTexture) ?? null;
        if (parameter === this.TEXTURE_BINDING_2D_ARRAY) return this.__texture2DArray.get(this.__activeTexture) ?? null;
        if (parameter === this.SAMPLER_BINDING) {
            return this.__samplers.get(this.__activeTexture - this.TEXTURE0) ?? null;
        }
        if (parameter === this.FRAMEBUFFER_BINDING) return this.__framebuffer;
        if (parameter === this.DRAW_FRAMEBUFFER_BINDING) return this.__drawFramebuffer;
        if (parameter === this.READ_FRAMEBUFFER_BINDING) return this.__readFramebuffer;
        if (parameter === this.RENDERBUFFER_BINDING) return this.__renderbuffer;
        if (parameter === this.ARRAY_BUFFER_BINDING) return this.__arrayBuffer;
        if (parameter === this.ELEMENT_ARRAY_BUFFER_BINDING) return this.__elementArrayBuffer;
        if (parameter === this.UNIFORM_BUFFER_BINDING) return this.__uniformBuffer;
        if (parameter === this.COPY_READ_BUFFER_BINDING) return this.__copyReadBuffer;
        if (parameter === this.COPY_WRITE_BUFFER_BINDING) return this.__copyWriteBuffer;
        if (parameter === this.PIXEL_PACK_BUFFER_BINDING) return this.__pixelPackBuffer;
        if (parameter === this.PIXEL_UNPACK_BUFFER_BINDING) return this.__pixelUnpackBuffer;
        if (parameter === this.CURRENT_PROGRAM) return this.__currentProgram;
        if (parameter === this.UNPACK_ALIGNMENT) return this.__unpackAlignment;
        if (parameter === this.MAX_CLIENT_WAIT_TIMEOUT_WEBGL) return 0;
        if (parameter === this.TRANSFORM_FEEDBACK_BINDING) return this.__transformFeedback;
        if (parameter === 0x85B5) return this.__vertexArray;
        if (parameter === this.TRANSFORM_FEEDBACK_ACTIVE) return this.__transformFeedbackActive;
        if (parameter === this.TRANSFORM_FEEDBACK_PAUSED) return this.__transformFeedbackPaused;
        if ([this.BLEND, this.CULL_FACE, this.DEPTH_TEST, this.DITHER,
            this.POLYGON_OFFSET_FILL, this.SAMPLE_ALPHA_TO_COVERAGE,
            this.SAMPLE_COVERAGE, this.SCISSOR_TEST, this.STENCIL_TEST,
            this.RASTERIZER_DISCARD, this.DEPTH_WRITEMASK,
            this.SAMPLE_COVERAGE_INVERT, 0x864F, 0x2A02].includes(parameter)) {
            return call("webglGetBoolean", this.canvas, parameter);
        }
        if (parameter === this.COLOR_WRITEMASK) {
            const mask = call("webglGetBoolean4Mask", this.canvas, parameter);
            return [0, 1, 2, 3].map(index => Boolean(mask & (1 << index)));
        }
        if ([this.DEPTH_CLEAR_VALUE, this.LINE_WIDTH, this.POLYGON_OFFSET_FACTOR,
            0x84FF,
            this.POLYGON_OFFSET_UNITS, this.SAMPLE_COVERAGE_VALUE, 0x8E1B,
            0x8E5B, 0x8E5C].includes(parameter)) {
            return call("webglGetFloat", this.canvas, parameter);
        }
        if ([this.ALIASED_LINE_WIDTH_RANGE, this.ALIASED_POINT_SIZE_RANGE,
            this.DEPTH_RANGE].includes(parameter)) {
            return call("webglGetFloat2", this.canvas, parameter);
        }
        if ([this.BLEND_COLOR, this.COLOR_CLEAR_VALUE].includes(parameter)) {
            return call("webglGetFloat4", this.canvas, parameter);
        }
        if (parameter === this.MAX_VIEWPORT_DIMS) {
            return call("webglGetInteger2", this.canvas, parameter);
        }
        if ([this.SCISSOR_BOX, this.VIEWPORT].includes(parameter)) {
            return call("webglGetInteger4", this.canvas, parameter);
        }
        return call("webglGetInteger", this.canvas, parameter);
    }
    getSupportedExtensions() {
        if (this.__contextLost) return null;
        return [
            "WEBGL_debug_renderer_info",
            "WEBGL_lose_context",
            ...JSON.parse(call("webglSupportedExtensions", this.canvas)),
        ];
    }
    getExtension(name) {
        name = String(name).toLowerCase();
        if (name === "webgl_lose_context") return this.__lossExtension;
        if (this.__contextLost) return null;
        const canonical = this.getSupportedExtensions()
            .find(extension => extension.toLowerCase() === name);
        if (canonical === undefined) return null;
        if (this.__extensions.has(canonical)) return this.__extensions.get(canonical);
        if (canonical !== "WEBGL_debug_renderer_info") {
            call("webglEnableWebExtension", this.canvas, canonical);
        }
        let extension;
        switch (canonical) {
            case "WEBGL_debug_renderer_info":
                extension = Object.freeze({ UNMASKED_VENDOR_WEBGL: 0x9245, UNMASKED_RENDERER_WEBGL: 0x9246 });
                break;
            case "OES_vertex_array_object":
                extension = new OESVertexArrayObject(construct, this);
                break;
            case "OES_standard_derivatives":
                extension = Object.freeze({ FRAGMENT_SHADER_DERIVATIVE_HINT_OES: 0x8B8B });
                break;
            case "EXT_blend_minmax":
                extension = Object.freeze({ MIN_EXT: 0x8007, MAX_EXT: 0x8008 });
                break;
            case "WEBGL_debug_shaders":
                extension = new WEBGLDebugShaders(construct, this);
                break;
            case "KHR_parallel_shader_compile":
                extension = Object.freeze({ COMPLETION_STATUS_KHR: 0x91B1 });
                break;
            case "EXT_clip_control":
                extension = new EXTClipControl(construct, this);
                break;
            case "EXT_polygon_offset_clamp":
                extension = new EXTPolygonOffsetClamp(construct, this);
                break;
            case "EXT_depth_clamp":
                extension = Object.freeze({ DEPTH_CLAMP_EXT: 0x864F });
                break;
            case "EXT_texture_mirror_clamp_to_edge":
                extension = Object.freeze({ MIRROR_CLAMP_TO_EDGE_EXT: 0x8743 });
                break;
            case "EXT_texture_norm16":
                extension = Object.freeze({
                    R16_EXT: 0x822A,
                    RG16_EXT: 0x822C,
                    RGB16_EXT: 0x8054,
                    RGBA16_EXT: 0x805B,
                    R16_SNORM_EXT: 0x8F98,
                    RG16_SNORM_EXT: 0x8F99,
                    RGB16_SNORM_EXT: 0x8F9A,
                    RGBA16_SNORM_EXT: 0x8F9B,
                });
                break;
            case "EXT_render_snorm":
                extension = Object.freeze({});
                break;
            case "OES_shader_multisample_interpolation":
                extension = Object.freeze({
                    MIN_FRAGMENT_INTERPOLATION_OFFSET_OES: 0x8E5B,
                    MAX_FRAGMENT_INTERPOLATION_OFFSET_OES: 0x8E5C,
                    FRAGMENT_INTERPOLATION_OFFSET_BITS_OES: 0x8E5D,
                });
                break;
            case "WEBGL_clip_cull_distance":
                extension = Object.freeze({
                    MAX_CLIP_DISTANCES_WEBGL: 0x0D32,
                    MAX_CULL_DISTANCES_WEBGL: 0x82F9,
                    MAX_COMBINED_CLIP_AND_CULL_DISTANCES_WEBGL: 0x82FA,
                    CLIP_DISTANCE0_WEBGL: 0x3000,
                    CLIP_DISTANCE1_WEBGL: 0x3001,
                    CLIP_DISTANCE2_WEBGL: 0x3002,
                    CLIP_DISTANCE3_WEBGL: 0x3003,
                    CLIP_DISTANCE4_WEBGL: 0x3004,
                    CLIP_DISTANCE5_WEBGL: 0x3005,
                    CLIP_DISTANCE6_WEBGL: 0x3006,
                    CLIP_DISTANCE7_WEBGL: 0x3007,
                });
                break;
            case "WEBGL_provoking_vertex":
                extension = new WEBGLProvokingVertex(construct, this);
                break;
            case "WEBGL_stencil_texturing":
                extension = Object.freeze({
                    DEPTH_STENCIL_TEXTURE_MODE_WEBGL: 0x90EA,
                    STENCIL_INDEX_WEBGL: 0x1901,
                });
                break;
            case "WEBGL_render_shared_exponent":
                extension = Object.freeze({});
                break;
            case "OES_texture_half_float":
                extension = Object.freeze({ HALF_FLOAT_OES: 0x8D61 });
                break;
            case "WEBGL_depth_texture":
                extension = Object.freeze({ UNSIGNED_INT_24_8_WEBGL: 0x84FA });
                break;
            case "WEBGL_color_buffer_float":
                this.getExtension("OES_texture_float");
                extension = Object.freeze({
                    RGBA32F_EXT: 0x8814,
                    FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT: 0x8211,
                    UNSIGNED_NORMALIZED_EXT: 0x8C17,
                });
                break;
            case "EXT_color_buffer_half_float":
                if (this.__version === 1) this.getExtension("OES_texture_half_float");
                extension = Object.freeze({
                    RGBA16F_EXT: 0x881A,
                    RGB16F_EXT: 0x881B,
                    FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT: 0x8211,
                    UNSIGNED_NORMALIZED_EXT: 0x8C17,
                });
                break;
            case "EXT_color_buffer_float":
            case "EXT_float_blend":
                extension = Object.freeze({});
                break;
            case "ANGLE_instanced_arrays":
                extension = new ANGLEInstancedArrays(construct, this);
                break;
            case "WEBGL_draw_buffers":
                extension = new WEBGLDrawBuffers(construct, this);
                break;
            case "WEBGL_compressed_texture_etc":
                extension = compressedTextureExtension(this, {
                    COMPRESSED_R11_EAC: 0x9270,
                    COMPRESSED_SIGNED_R11_EAC: 0x9271,
                    COMPRESSED_RG11_EAC: 0x9272,
                    COMPRESSED_SIGNED_RG11_EAC: 0x9273,
                    COMPRESSED_RGB8_ETC2: 0x9274,
                    COMPRESSED_SRGB8_ETC2: 0x9275,
                    COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2: 0x9276,
                    COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2: 0x9277,
                    COMPRESSED_RGBA8_ETC2_EAC: 0x9278,
                    COMPRESSED_SRGB8_ALPHA8_ETC2_EAC: 0x9279,
                });
                break;
            case "EXT_sRGB":
                extension = Object.freeze({
                    SRGB_EXT: 0x8C40,
                    SRGB_ALPHA_EXT: 0x8C42,
                    SRGB8_ALPHA8_EXT: 0x8C43,
                    FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING_EXT: 0x8210,
                });
                break;
            case "WEBGL_compressed_texture_etc1":
                extension = compressedTextureExtension(this, {
                    COMPRESSED_RGB_ETC1_WEBGL: 0x8D64,
                });
                break;
            case "OES_fbo_render_mipmap":
                extension = Object.freeze({});
                break;
            case "WEBGL_blend_func_extended":
                extension = Object.freeze({
                    SRC1_COLOR_WEBGL: 0x88F9,
                    SRC1_ALPHA_WEBGL: 0x8589,
                    ONE_MINUS_SRC1_COLOR_WEBGL: 0x88FA,
                    ONE_MINUS_SRC1_ALPHA_WEBGL: 0x88FB,
                    MAX_DUAL_SOURCE_DRAW_BUFFERS_WEBGL: 0x88FC,
                });
                break;
            case "WEBGL_polygon_mode":
                extension = new WEBGLPolygonMode(construct, this);
                break;
            case "EXT_texture_filter_anisotropic":
                extension = Object.freeze({
                    TEXTURE_MAX_ANISOTROPY_EXT: 0x84FE,
                    MAX_TEXTURE_MAX_ANISOTROPY_EXT: 0x84FF,
                });
                break;
            case "WEBGL_compressed_texture_s3tc":
                extension = compressedTextureExtension(this, {
                    COMPRESSED_RGB_S3TC_DXT1_EXT: 0x83F0,
                    COMPRESSED_RGBA_S3TC_DXT1_EXT: 0x83F1,
                    COMPRESSED_RGBA_S3TC_DXT3_EXT: 0x83F2,
                    COMPRESSED_RGBA_S3TC_DXT5_EXT: 0x83F3,
                });
                break;
            case "WEBGL_compressed_texture_s3tc_srgb":
                extension = compressedTextureExtension(this, {
                    COMPRESSED_SRGB_S3TC_DXT1_EXT: 0x8C4C,
                    COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT: 0x8C4D,
                    COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT: 0x8C4E,
                    COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT: 0x8C4F,
                });
                break;
            case "EXT_texture_compression_bptc":
                extension = compressedTextureExtension(this, {
                    COMPRESSED_RGBA_BPTC_UNORM_EXT: 0x8E8C,
                    COMPRESSED_SRGB_ALPHA_BPTC_UNORM_EXT: 0x8E8D,
                    COMPRESSED_RGB_BPTC_SIGNED_FLOAT_EXT: 0x8E8E,
                    COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_EXT: 0x8E8F,
                });
                break;
            case "EXT_texture_compression_rgtc":
                extension = compressedTextureExtension(this, {
                    COMPRESSED_RED_RGTC1_EXT: 0x8DBB,
                    COMPRESSED_SIGNED_RED_RGTC1_EXT: 0x8DBC,
                    COMPRESSED_RED_GREEN_RGTC2_EXT: 0x8DBD,
                    COMPRESSED_SIGNED_RED_GREEN_RGTC2_EXT: 0x8DBE,
                });
                break;
            case "WEBGL_compressed_texture_astc": {
                extension = new WEBGLCompressedTextureASTC(construct, this);
                for (const [name, format] of Object.entries(extension)) {
                    if (name.startsWith("COMPRESSED_")) this.__compressedTextureFormats.add(format);
                }
                Object.freeze(extension);
                break;
            }
            case "WEBGL_compressed_texture_pvrtc":
                extension = compressedTextureExtension(this, {
                    COMPRESSED_RGB_PVRTC_4BPPV1_IMG: 0x8C00,
                    COMPRESSED_RGB_PVRTC_2BPPV1_IMG: 0x8C01,
                    COMPRESSED_RGBA_PVRTC_4BPPV1_IMG: 0x8C02,
                    COMPRESSED_RGBA_PVRTC_2BPPV1_IMG: 0x8C03,
                });
                break;
            case "EXT_disjoint_timer_query":
                extension = new EXTDisjointTimerQuery(construct, this);
                break;
            case "EXT_disjoint_timer_query_webgl2":
                extension = new EXTDisjointTimerQueryWebGL2(construct, this);
                break;
            case "OES_draw_buffers_indexed":
                extension = new OESDrawBuffersIndexed(construct, this);
                break;
            case "WEBGL_multi_draw":
                extension = new WEBGLMultiDraw(construct, this);
                break;
            case "WEBGL_draw_instanced_base_vertex_base_instance":
                extension = new WEBGLDrawInstancedBaseVertexBaseInstance(construct, this);
                break;
            case "WEBGL_multi_draw_instanced_base_vertex_base_instance":
                this.getExtension("WEBGL_multi_draw");
                extension = new WEBGLMultiDrawInstancedBaseVertexBaseInstance(construct, this);
                break;
            default:
                extension = Object.freeze({});
                break;
        }
        this.__extensions.set(canonical, extension);
        return extension;
    }
    readPixels(x, y, width, height, format, type, destination, destinationOffset = 0) {
        if (this.__contextLost) return;
        format = Number(format) >>> 0;
        type = Number(type) >>> 0;
        width = Number(width) | 0;
        height = Number(height) | 0;
        if (typeof destination === "number") {
            if (this.__version !== 2 || this.__pixelPackBuffer === null) {
                throw new TypeError("A pixel pack buffer must be bound for offset readback");
            }
            const offset = Number(destination) | 0;
            if (offset < 0) throw new RangeError("Pixel pack buffer offset must be non-negative");
            call("webglReadPixelsOffset", this.canvas, Number(x), Number(y), width, height,
                format, type, offset);
            return;
        }
        if (!ArrayBuffer.isView(destination)) throw new TypeError("A typed-array readback destination is required");
        const destinationTypes = new Map([
            [this.BYTE, Int8Array], [this.UNSIGNED_BYTE, Uint8Array],
            [this.SHORT, Int16Array], [this.UNSIGNED_SHORT, Uint16Array],
            [this.INT, Int32Array], [this.UNSIGNED_INT, Uint32Array],
            [this.FLOAT, Float32Array], [this.HALF_FLOAT, Uint16Array],
            [this.UNSIGNED_SHORT_5_6_5, Uint16Array],
            [this.UNSIGNED_SHORT_4_4_4_4, Uint16Array],
            [this.UNSIGNED_SHORT_5_5_5_1, Uint16Array],
            [this.UNSIGNED_INT_2_10_10_10_REV, Uint32Array],
            [this.UNSIGNED_INT_10F_11F_11F_REV, Uint32Array],
            [this.UNSIGNED_INT_5_9_9_9_REV, Uint32Array],
            [this.UNSIGNED_INT_24_8, Uint32Array],
        ]);
        const expectedType = destinationTypes.get(type);
        const clampedByte = type === this.UNSIGNED_BYTE && destination instanceof Uint8ClampedArray;
        if ((expectedType === undefined || !(destination instanceof expectedType)) && !clampedByte) {
            throw new TypeError("The readPixels format, type, and destination do not match");
        }
        destinationOffset = this.__version === 2 ? Number(destinationOffset) >>> 0 : 0;
        if (destinationOffset > destination.length) {
            throw new RangeError("The readPixels destination offset is outside the array");
        }
        const required = Math.max(0, width) * Math.max(0, height)
            * texturePixelBytes(this, format, type);
        const byteOffset = destination.byteOffset + destinationOffset * destination.BYTES_PER_ELEMENT;
        if (destination.byteLength - destinationOffset * destination.BYTES_PER_ELEMENT < required) {
            throw new RangeError("The readPixels destination is too small");
        }
        const bytes = call("webglReadPixels", this.canvas, Number(x), Number(y), width, height,
            format, type);
        new Uint8Array(destination.buffer, byteOffset, required).set(bytes.subarray(0, required));
    }
    getContextAttributes() {
        if (this.__contextLost) return null;
        return { alpha: true, antialias: false, depth: true, desynchronized: false, failIfMajorPerformanceCaveat: false, powerPreference: "default", premultipliedAlpha: true, preserveDrawingBuffer: false, stencil: true, xrCompatible: false };
    }
    isContextLost() { return this.__contextLost; }
    getError() {
        if (!this.__contextLost) {
            return this.__errors.shift() ?? call("webglGetError", this.canvas);
        }
        if (!this.__lostError) return this.NO_ERROR;
        this.__lostError = false;
        return this.CONTEXT_LOST_WEBGL;
    }
}

class WebGL2RenderingContext extends WebGLRenderingContext {
    constructor(token, canvas) { super(token, canvas, 2); }
    uniform1ui(location, x) { setUniformU(this, location, 1, [x]); }
    uniform2ui(location, x, y) { setUniformU(this, location, 2, [x, y]); }
    uniform3ui(location, x, y, z) { setUniformU(this, location, 3, [x, y, z]); }
    uniform4ui(location, x, y, z, w) { setUniformU(this, location, 4, [x, y, z, w]); }
    uniform1uiv(location, value, sourceOffset = 0, sourceLength = 0) {
        setUniformU(this, location, 1, uniformValues(value, Uint32Array, 1, "uniform1uiv", sourceOffset, sourceLength));
    }
    uniform2uiv(location, value, sourceOffset = 0, sourceLength = 0) {
        setUniformU(this, location, 2, uniformValues(value, Uint32Array, 2, "uniform2uiv", sourceOffset, sourceLength));
    }
    uniform3uiv(location, value, sourceOffset = 0, sourceLength = 0) {
        setUniformU(this, location, 3, uniformValues(value, Uint32Array, 3, "uniform3uiv", sourceOffset, sourceLength));
    }
    uniform4uiv(location, value, sourceOffset = 0, sourceLength = 0) {
        setUniformU(this, location, 4, uniformValues(value, Uint32Array, 4, "uniform4uiv", sourceOffset, sourceLength));
    }
    uniformMatrix2x3fv(location, transpose, value, sourceOffset = 0, sourceLength = 0) {
        setUniformMatrixRect(this, location, 2, 3, transpose, value, "uniformMatrix2x3fv", sourceOffset, sourceLength);
    }
    uniformMatrix3x2fv(location, transpose, value, sourceOffset = 0, sourceLength = 0) {
        setUniformMatrixRect(this, location, 3, 2, transpose, value, "uniformMatrix3x2fv", sourceOffset, sourceLength);
    }
    uniformMatrix2x4fv(location, transpose, value, sourceOffset = 0, sourceLength = 0) {
        setUniformMatrixRect(this, location, 2, 4, transpose, value, "uniformMatrix2x4fv", sourceOffset, sourceLength);
    }
    uniformMatrix4x2fv(location, transpose, value, sourceOffset = 0, sourceLength = 0) {
        setUniformMatrixRect(this, location, 4, 2, transpose, value, "uniformMatrix4x2fv", sourceOffset, sourceLength);
    }
    uniformMatrix3x4fv(location, transpose, value, sourceOffset = 0, sourceLength = 0) {
        setUniformMatrixRect(this, location, 3, 4, transpose, value, "uniformMatrix3x4fv", sourceOffset, sourceLength);
    }
    uniformMatrix4x3fv(location, transpose, value, sourceOffset = 0, sourceLength = 0) {
        setUniformMatrixRect(this, location, 4, 3, transpose, value, "uniformMatrix4x3fv", sourceOffset, sourceLength);
    }
    vertexAttribIPointer(index, size, type, stride, offset) {
        call("webglVertexAttribIPointer", this.canvas, Number(index) >>> 0, Number(size) | 0,
            Number(type) >>> 0, Number(stride) | 0, Number(offset) | 0);
    }
    vertexAttribI4i(index, x, y, z, w) {
        index = Number(index) >>> 0;
        const values = new Int32Array([x, y, z, w]);
        call("webglVertexAttribI4i", this.canvas, index, values);
        this.__currentAttributes.set(index, values);
    }
    vertexAttribI4iv(index, values) {
        const data = values instanceof Int32Array ? values : new Int32Array(values);
        if (data.length < 4) throw new RangeError("vertexAttribI4iv requires four values");
        this.vertexAttribI4i(index, data[0], data[1], data[2], data[3]);
    }
    vertexAttribI4ui(index, x, y, z, w) {
        index = Number(index) >>> 0;
        const values = new Uint32Array([x, y, z, w]);
        call("webglVertexAttribI4ui", this.canvas, index, values);
        this.__currentAttributes.set(index, values);
    }
    vertexAttribI4uiv(index, values) {
        const data = values instanceof Uint32Array ? values : new Uint32Array(values);
        if (data.length < 4) throw new RangeError("vertexAttribI4uiv requires four values");
        this.vertexAttribI4ui(index, data[0], data[1], data[2], data[3]);
    }
    drawRangeElements(mode, start, end, count, type, offset) {
        call("webglDrawRangeElements", this.canvas, Number(mode) >>> 0, Number(start) >>> 0,
            Number(end) >>> 0, Number(count) | 0, Number(type) >>> 0, Number(offset) | 0);
    }
    copyBufferSubData(readTarget, writeTarget, readOffset, writeOffset, size) {
        call("webglCopyBufferSubData", this.canvas, Number(readTarget) >>> 0,
            Number(writeTarget) >>> 0, Number(readOffset) | 0, Number(writeOffset) | 0,
            Number(size) | 0);
    }
    getFragDataLocation(program, name) {
        return call("webglGetFragDataLocation", this.canvas,
            objectId(this, program, WebGLProgram), String(name));
    }
    getUniformIndices(program, uniformNames) {
        return JSON.parse(call("webglGetUniformIndices", this.canvas,
            objectId(this, program, WebGLProgram), JSON.stringify(Array.from(uniformNames, String))));
    }
    getActiveUniforms(program, uniformIndices, parameter) {
        return JSON.parse(call("webglGetActiveUniforms", this.canvas,
            objectId(this, program, WebGLProgram),
            new Uint32Array(Array.from(uniformIndices, value => Number(value) >>> 0)),
            Number(parameter) >>> 0));
    }
    getUniformBlockIndex(program, uniformBlockName) {
        return call("webglGetUniformBlockIndex", this.canvas,
            objectId(this, program, WebGLProgram), String(uniformBlockName));
    }
    getActiveUniformBlockParameter(program, uniformBlockIndex, parameter) {
        const value = JSON.parse(call("webglGetActiveUniformBlockParameter", this.canvas,
            objectId(this, program, WebGLProgram), Number(uniformBlockIndex) >>> 0,
            Number(parameter) >>> 0));
        return Number(parameter) === this.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES
            ? new Uint32Array(value) : value;
    }
    getActiveUniformBlockName(program, uniformBlockIndex) {
        return call("webglGetActiveUniformBlockName", this.canvas,
            objectId(this, program, WebGLProgram), Number(uniformBlockIndex) >>> 0);
    }
    uniformBlockBinding(program, uniformBlockIndex, uniformBlockBinding) {
        call("webglUniformBlockBinding", this.canvas, objectId(this, program, WebGLProgram),
            Number(uniformBlockIndex) >>> 0, Number(uniformBlockBinding) >>> 0);
    }
    texStorage2D(target, levels, internalFormat, width, height) {
        call(
            "webglTexStorage2D",
            this.canvas,
            Number(target) >>> 0,
            Number(levels) | 0,
            Number(internalFormat) >>> 0,
            Number(width) | 0,
            Number(height) | 0,
        );
    }
    texStorage3D(target, levels, internalFormat, width, height, depth) {
        call(
            "webglTexStorage3D",
            this.canvas,
            Number(target) >>> 0,
            Number(levels) | 0,
            Number(internalFormat) >>> 0,
            Number(width) | 0,
            Number(height) | 0,
            Number(depth) | 0,
        );
    }
    bindBufferBase(target, index, buffer) {
        target = Number(target) >>> 0;
        index = Number(index) >>> 0;
        call(
            "webglBindBufferBase",
            this.canvas,
            target,
            index,
            objectId(this, buffer, WebGLBuffer, true),
        );
        this.__indexedBuffers.set(`${target}:${index}`, { buffer, offset: 0, size: 0 });
    }
    bindBufferRange(target, index, buffer, offset, size) {
        target = Number(target) >>> 0;
        index = Number(index) >>> 0;
        offset = Number(offset) | 0;
        size = Number(size) | 0;
        call(
            "webglBindBufferRange",
            this.canvas,
            target,
            index,
            objectId(this, buffer, WebGLBuffer, true),
            offset,
            size,
        );
        this.__indexedBuffers.set(`${target}:${index}`, { buffer, offset, size });
    }
    getIndexedParameter(target, index) {
        target = Number(target) >>> 0;
        index = Number(index) >>> 0;
        if (target === this.COLOR_WRITEMASK) {
            const mask = call("webglGetIndexedColorMask", this.canvas, index);
            return [0, 1, 2, 3].map(component => Boolean(mask & (1 << component)));
        }
        if ([this.BLEND_EQUATION_RGB, this.BLEND_EQUATION_ALPHA,
            this.BLEND_SRC_RGB, this.BLEND_SRC_ALPHA,
            this.BLEND_DST_RGB, this.BLEND_DST_ALPHA].includes(target)) {
            return call("webglGetIndexedParameterI", this.canvas, target, index);
        }
        const bindingTarget = target === this.TRANSFORM_FEEDBACK_BUFFER_BINDING
            || target === this.TRANSFORM_FEEDBACK_BUFFER_START
            || target === this.TRANSFORM_FEEDBACK_BUFFER_SIZE
            ? this.TRANSFORM_FEEDBACK_BUFFER
            : target === this.UNIFORM_BUFFER_BINDING
                || target === this.UNIFORM_BUFFER_START
                || target === this.UNIFORM_BUFFER_SIZE
                ? this.UNIFORM_BUFFER
            : target;
        const binding = this.__indexedBuffers.get(`${bindingTarget}:${index}`)
            ?? { buffer: null, offset: 0, size: 0 };
        if (target === this.TRANSFORM_FEEDBACK_BUFFER_BINDING) return binding.buffer;
        if (target === this.TRANSFORM_FEEDBACK_BUFFER_START) return binding.offset;
        if (target === this.TRANSFORM_FEEDBACK_BUFFER_SIZE) return binding.size;
        if (target === this.UNIFORM_BUFFER_BINDING) return binding.buffer;
        if (target === this.UNIFORM_BUFFER_START) return binding.offset;
        if (target === this.UNIFORM_BUFFER_SIZE) return binding.size;
        return null;
    }
    getBufferSubData(target, sourceByteOffset, destination, destinationOffset = 0, length = 0) {
        if (!ArrayBuffer.isView(destination)) throw new TypeError("Destination must be an ArrayBufferView");
        destinationOffset = Number(destinationOffset) >>> 0;
        length = Number(length) >>> 0;
        const elementBytes = destination.BYTES_PER_ELEMENT ?? 1;
        const elements = destination.byteLength / elementBytes;
        if (destinationOffset > elements) throw new RangeError("Destination offset is out of range");
        if (length === 0) length = elements - destinationOffset;
        if (destinationOffset + length > elements) throw new RangeError("Destination range is out of bounds");
        const byteOffset = destination.byteOffset + destinationOffset * elementBytes;
        const byteLength = length * elementBytes;
        const result = call(
            "webglGetBufferSubData",
            this.canvas,
            Number(target) >>> 0,
            Number(sourceByteOffset) | 0,
            byteLength,
        );
        new Uint8Array(destination.buffer, byteOffset, byteLength).set(result);
    }
    transformFeedbackVaryings(program, varyings, bufferMode) {
        call(
            "webglTransformFeedbackVaryings",
            this.canvas,
            objectId(this, program, WebGLProgram),
            JSON.stringify(Array.from(varyings, String)),
            Number(bufferMode) >>> 0,
        );
    }
    getTransformFeedbackVarying(program, index) {
        const encoded = call(
            "webglGetTransformFeedbackVarying",
            this.canvas,
            objectId(this, program, WebGLProgram),
            Number(index) >>> 0,
        );
        if (encoded === null) return null;
        const info = JSON.parse(encoded);
        return new WebGLActiveInfo(construct, info.name, info.size, info.type);
    }
    createTransformFeedback() {
        if (this.__contextLost) return null;
        return new WebGLTransformFeedback(
            construct,
            this,
            call("webglCreateTransformFeedback", this.canvas),
        );
    }
    bindTransformFeedback(target, feedback) {
        target = Number(target) >>> 0;
        call(
            "webglBindTransformFeedback",
            this.canvas,
            target,
            objectId(this, feedback, WebGLTransformFeedback, true),
        );
        if (target === this.TRANSFORM_FEEDBACK) this.__transformFeedback = feedback;
    }
    beginTransformFeedback(primitiveMode) {
        call("webglBeginTransformFeedback", this.canvas, Number(primitiveMode) >>> 0);
        this.__transformFeedbackActive = true;
        this.__transformFeedbackPaused = false;
    }
    endTransformFeedback() {
        call("webglEndTransformFeedback", this.canvas);
        this.__transformFeedbackActive = false;
        this.__transformFeedbackPaused = false;
    }
    pauseTransformFeedback() {
        call("webglPauseTransformFeedback", this.canvas);
        this.__transformFeedbackPaused = true;
    }
    resumeTransformFeedback() {
        call("webglResumeTransformFeedback", this.canvas);
        this.__transformFeedbackPaused = false;
    }
    deleteTransformFeedback(feedback) {
        if (feedback === null || feedback?.__id === 0) return;
        call(
            "webglDeleteTransformFeedback",
            this.canvas,
            objectId(this, feedback, WebGLTransformFeedback),
        );
        if (this.__transformFeedback === feedback) this.__transformFeedback = null;
        feedback.__id = 0;
    }
    isTransformFeedback(feedback) {
        return feedback instanceof WebGLTransformFeedback
            && feedback.__context === this && feedback.__id !== 0;
    }
    texImage3D(target, level, internalFormat, width, height, depth, border, format, type, pixels,
        sourceOffset = 0) {
        target = Number(target) >>> 0;
        level = Number(level) | 0;
        internalFormat = Number(internalFormat) | 0;
        width = Number(width) | 0;
        height = Number(height) | 0;
        depth = Number(depth) | 0;
        border = Number(border) | 0;
        format = Number(format) >>> 0;
        type = Number(type) >>> 0;
        if (typeof pixels === "number") {
            if (this.__pixelUnpackBuffer === null) {
                throw new TypeError("A pixel unpack buffer must be bound for offset upload");
            }
            const offset = Number(pixels) | 0;
            if (offset < 0) throw new RangeError("Pixel unpack buffer offset must be non-negative");
            call("webglTexImage3DOffset", this.canvas, target, level, internalFormat,
                width, height, depth, border, format, type, offset);
            return;
        }
        const rangedPixels = sourceOffset === 0
            ? pixels : bufferSourceBytes(this, pixels, sourceOffset, 0);
        const data = textureBytes(rangedPixels, true);
        const required = textureByteLength3D(this, width, height, depth, format, type);
        if (data !== null && data.byteLength < required) throw new RangeError("Texture pixel data is too small");
        call("webglTexImage3D", this.canvas, target, level, internalFormat, width, height, depth, border, format, type, data !== null, data ?? new Uint8Array());
    }
    texSubImage3D(target, level, xoffset, yoffset, zoffset, width, height, depth, format, type, pixels,
        sourceOffset = 0) {
        target = Number(target) >>> 0;
        level = Number(level) | 0;
        xoffset = Number(xoffset) | 0;
        yoffset = Number(yoffset) | 0;
        zoffset = Number(zoffset) | 0;
        width = Number(width) | 0;
        height = Number(height) | 0;
        depth = Number(depth) | 0;
        format = Number(format) >>> 0;
        type = Number(type) >>> 0;
        if (typeof pixels === "number") {
            if (this.__pixelUnpackBuffer === null) {
                throw new TypeError("A pixel unpack buffer must be bound for offset upload");
            }
            const offset = Number(pixels) | 0;
            if (offset < 0) throw new RangeError("Pixel unpack buffer offset must be non-negative");
            call("webglTexSubImage3DOffset", this.canvas, target, level, xoffset, yoffset, zoffset,
                width, height, depth, format, type, offset);
            return;
        }
        const rangedPixels = sourceOffset === 0
            ? pixels : bufferSourceBytes(this, pixels, sourceOffset, 0);
        const data = textureBytes(rangedPixels);
        if (data.byteLength < textureByteLength3D(this, width, height, depth, format, type)) {
            throw new RangeError("Texture pixel data is too small");
        }
        call("webglTexSubImage3D", this.canvas, target, level, xoffset, yoffset, zoffset, width, height, depth, format, type, data);
    }
    compressedTexImage3D(target, level, internalFormat, width, height, depth, border, dataOrSize,
        sourceOffset = 0, sourceLengthOverride = 0) {
        target = Number(target) >>> 0;
        level = Number(level) | 0;
        internalFormat = Number(internalFormat) >>> 0;
        width = Number(width) | 0;
        height = Number(height) | 0;
        depth = Number(depth) | 0;
        border = Number(border) | 0;
        if (typeof dataOrSize === "number") {
            if (this.__pixelUnpackBuffer === null) {
                throw new TypeError("A pixel unpack buffer must be bound for compressed offset upload");
            }
            call("webglCompressedTexImage3DOffset", this.canvas, target, level, internalFormat,
                width, height, depth, border, Number(dataOrSize) | 0,
                Number(sourceOffset) >>> 0);
            return;
        }
        const data = compressedSourceBytes(
            this, dataOrSize, sourceOffset, sourceLengthOverride);
        call("webglCompressedTexImage3D", this.canvas, target, level, internalFormat,
            width, height, depth, border, data);
    }
    compressedTexSubImage3D(target, level, xoffset, yoffset, zoffset, width, height, depth,
        format, dataOrSize, sourceOffset = 0, sourceLengthOverride = 0) {
        target = Number(target) >>> 0;
        level = Number(level) | 0;
        xoffset = Number(xoffset) | 0;
        yoffset = Number(yoffset) | 0;
        zoffset = Number(zoffset) | 0;
        width = Number(width) | 0;
        height = Number(height) | 0;
        depth = Number(depth) | 0;
        format = Number(format) >>> 0;
        if (typeof dataOrSize === "number") {
            if (this.__pixelUnpackBuffer === null) {
                throw new TypeError("A pixel unpack buffer must be bound for compressed offset upload");
            }
            call("webglCompressedTexSubImage3DOffset", this.canvas, target, level,
                xoffset, yoffset, zoffset, width, height, depth, format,
                Number(dataOrSize) >>> 0, Number(sourceOffset) >>> 0);
            return;
        }
        const data = compressedSourceBytes(
            this, dataOrSize, sourceOffset, sourceLengthOverride);
        call("webglCompressedTexSubImage3D", this.canvas, target, level,
            xoffset, yoffset, zoffset, width, height, depth, format, data);
    }
    copyTexSubImage3D(target, level, xoffset, yoffset, zoffset, x, y, width, height) {
        call(
            "webglCopyTexSubImage3D",
            this.canvas,
            Number(target) >>> 0,
            Number(level) | 0,
            Number(xoffset) | 0,
            Number(yoffset) | 0,
            Number(zoffset) | 0,
            Number(x) | 0,
            Number(y) | 0,
            Number(width) | 0,
            Number(height) | 0,
        );
    }
    createSampler() {
        if (this.__contextLost) return null;
        return new WebGLSampler(construct, this, call("webglCreateSampler", this.canvas));
    }
    bindSampler(unit, sampler) {
        unit = Number(unit) >>> 0;
        call(
            "webglBindSampler",
            this.canvas,
            unit,
            objectId(this, sampler, WebGLSampler, true),
        );
        this.__samplers.set(unit, sampler);
    }
    samplerParameteri(sampler, parameter, value) {
        call(
            "webglSamplerParameteri",
            this.canvas,
            objectId(this, sampler, WebGLSampler),
            Number(parameter) >>> 0,
            Number(value) | 0,
        );
    }
    samplerParameterf(sampler, parameter, value) {
        call(
            "webglSamplerParameterf",
            this.canvas,
            objectId(this, sampler, WebGLSampler),
            Number(parameter) >>> 0,
            Number(value),
        );
    }
    getSamplerParameter(sampler, parameter) {
        parameter = Number(parameter) >>> 0;
        const operation = parameter === this.TEXTURE_MIN_LOD || parameter === this.TEXTURE_MAX_LOD
            ? "webglGetSamplerParameterF"
            : "webglGetSamplerParameterI";
        return call(
            operation,
            this.canvas,
            objectId(this, sampler, WebGLSampler),
            parameter,
        );
    }
    deleteSampler(sampler) {
        if (sampler === null || sampler?.__id === 0) return;
        call("webglDeleteSampler", this.canvas, objectId(this, sampler, WebGLSampler));
        for (const [unit, binding] of this.__samplers) {
            if (binding === sampler) this.__samplers.set(unit, null);
        }
        sampler.__id = 0;
    }
    isSampler(sampler) {
        return sampler instanceof WebGLSampler
            && sampler.__context === this && sampler.__id !== 0;
    }
    createQuery() {
        if (this.__contextLost) return null;
        return new WebGLQuery(construct, this, call("webglCreateQuery", this.canvas));
    }
    beginQuery(target, query) {
        target = Number(target) >>> 0;
        call(
            "webglBeginQuery",
            this.canvas,
            target,
            objectId(this, query, WebGLQuery),
        );
        this.__queries.set(target, query);
        query.__target = target;
    }
    endQuery(target) {
        target = Number(target) >>> 0;
        call("webglEndQuery", this.canvas, target);
        this.__queries.set(target, null);
    }
    getQuery(target, parameter) {
        target = Number(target) >>> 0;
        parameter = Number(parameter) >>> 0;
        if (parameter === this.CURRENT_QUERY) {
            return target === 0x8E28 ? null : this.__queries.get(target) ?? null;
        }
        if (parameter === 0x8864) {
            return call("webglGetQueryCounterBits", this.canvas, target);
        }
        return null;
    }
    getQueryParameter(query, parameter) {
        parameter = Number(parameter) >>> 0;
        const operation = parameter === this.QUERY_RESULT
            && [0x88BF, 0x8E28].includes(query.__target)
            ? "webglGetQueryParameter64" : "webglGetQueryParameter";
        const value = call(
            operation,
            this.canvas,
            objectId(this, query, WebGLQuery),
            parameter,
        );
        if (parameter === this.QUERY_RESULT_AVAILABLE) return Boolean(value);
        if (parameter === this.QUERY_RESULT
            && [this.ANY_SAMPLES_PASSED, this.ANY_SAMPLES_PASSED_CONSERVATIVE]
                .includes(query.__target)) {
            return Boolean(value);
        }
        return value;
    }
    deleteQuery(query) {
        if (query === null || query?.__id === 0) return;
        call("webglDeleteQuery", this.canvas, objectId(this, query, WebGLQuery));
        for (const [target, active] of this.__queries) {
            if (active === query) this.__queries.set(target, null);
        }
        query.__id = 0;
    }
    isQuery(query) {
        return query instanceof WebGLQuery
            && query.__context === this && query.__id !== 0;
    }
    fenceSync(condition, flags) {
        if (this.__contextLost) return null;
        return new WebGLSync(
            construct,
            this,
            call(
                "webglFenceSync",
                this.canvas,
                Number(condition) >>> 0,
                Number(flags) >>> 0,
            ),
        );
    }
    clientWaitSync(sync, flags, timeout) {
        flags = Number(flags) >>> 0;
        timeout = Number(timeout);
        if (timeout > 0) {
            this.__errors.push(this.INVALID_OPERATION);
            return this.WAIT_FAILED;
        }
        if (flags !== 0 && flags !== this.SYNC_FLUSH_COMMANDS_BIT) {
            this.__errors.push(this.INVALID_VALUE);
            return this.WAIT_FAILED;
        }
        return call(
            "webglClientWaitSync",
            this.canvas,
            objectId(this, sync, WebGLSync),
            flags,
            timeout,
        );
    }
    waitSync(sync, flags, timeout) {
        flags = Number(flags) >>> 0;
        timeout = Number(timeout);
        if (flags !== 0 || timeout !== this.TIMEOUT_IGNORED) {
            this.__errors.push(this.INVALID_VALUE);
            return;
        }
        call(
            "webglWaitSync",
            this.canvas,
            objectId(this, sync, WebGLSync),
            flags,
            timeout,
        );
    }
    getSyncParameter(sync, parameter) {
        return call(
            "webglGetSyncParameter",
            this.canvas,
            objectId(this, sync, WebGLSync),
            Number(parameter) >>> 0,
        );
    }
    deleteSync(sync) {
        if (sync === null || sync?.__id === 0) return;
        call("webglDeleteSync", this.canvas, objectId(this, sync, WebGLSync));
        sync.__id = 0;
    }
    isSync(sync) {
        return sync instanceof WebGLSync
            && sync.__context === this && sync.__id !== 0;
    }
    framebufferTextureLayer(target, attachment, texture, level, layer) {
        call("webglFramebufferTextureLayer", this.canvas, Number(target) >>> 0, Number(attachment) >>> 0, objectId(this, texture, WebGLTexture, true), Number(level) | 0, Number(layer) | 0);
    }
    invalidateFramebuffer(target, attachments) {
        call("webglInvalidateFramebuffer", this.canvas, Number(target) >>> 0,
            new Uint32Array(Array.from(attachments, value => Number(value) >>> 0)));
    }
    invalidateSubFramebuffer(target, attachments, x, y, width, height) {
        call("webglInvalidateSubFramebuffer", this.canvas, Number(target) >>> 0,
            new Uint32Array(Array.from(attachments, value => Number(value) >>> 0)),
            Number(x) | 0, Number(y) | 0, Number(width) | 0, Number(height) | 0);
    }
    getInternalformatParameter(target, internalFormat, parameter) {
        return call("webglGetInternalformatParameter", this.canvas, Number(target) >>> 0,
            Number(internalFormat) >>> 0, Number(parameter) >>> 0);
    }
    clearBufferiv(buffer, drawBuffer, values, sourceOffset = 0) {
        const data = values instanceof Int32Array ? values : new Int32Array(values);
        sourceOffset = Number(sourceOffset) >>> 0;
        const length = Number(buffer) === this.COLOR ? 4 : 1;
        if (sourceOffset > data.length || length > data.length - sourceOffset) throw new RangeError("Clear value array is too small");
        call("webglClearBufferiv", this.canvas, Number(buffer) >>> 0, Number(drawBuffer) | 0,
            data.subarray(sourceOffset, sourceOffset + length));
    }
    clearBufferuiv(buffer, drawBuffer, values, sourceOffset = 0) {
        const data = values instanceof Uint32Array ? values : new Uint32Array(values);
        sourceOffset = Number(sourceOffset) >>> 0;
        const length = Number(buffer) === this.COLOR ? 4 : 1;
        if (sourceOffset > data.length || length > data.length - sourceOffset) throw new RangeError("Clear value array is too small");
        call("webglClearBufferuiv", this.canvas, Number(buffer) >>> 0, Number(drawBuffer) | 0,
            data.subarray(sourceOffset, sourceOffset + length));
    }
    clearBufferfv(buffer, drawBuffer, values, sourceOffset = 0) {
        const data = values instanceof Float32Array ? values : new Float32Array(values);
        sourceOffset = Number(sourceOffset) >>> 0;
        const length = Number(buffer) === this.COLOR ? 4 : 1;
        if (sourceOffset > data.length || length > data.length - sourceOffset) throw new RangeError("Clear value array is too small");
        call("webglClearBufferfv", this.canvas, Number(buffer) >>> 0, Number(drawBuffer) | 0,
            data.subarray(sourceOffset, sourceOffset + length));
    }
    clearBufferfi(buffer, drawBuffer, depth, stencil) {
        call("webglClearBufferfi", this.canvas, Number(buffer) >>> 0, Number(drawBuffer) | 0,
            Number(depth), Number(stencil) | 0);
    }
    renderbufferStorageMultisample(target, samples, internalFormat, width, height) {
        call("webglRenderbufferStorageMultisample", this.canvas, Number(target) >>> 0, Number(samples) | 0, Number(internalFormat) >>> 0, Number(width) | 0, Number(height) | 0);
    }
    drawBuffers(buffers) {
        call("webglDrawBuffers", this.canvas, new Uint32Array(Array.from(buffers, value => Number(value) >>> 0)));
    }
    readBuffer(source) { call("webglReadBuffer", this.canvas, Number(source) >>> 0); }
    blitFramebuffer(sourceX0, sourceY0, sourceX1, sourceY1, destinationX0, destinationY0, destinationX1, destinationY1, mask, filter) {
        call("webglBlitFramebuffer", this.canvas, Number(sourceX0) | 0, Number(sourceY0) | 0, Number(sourceX1) | 0, Number(sourceY1) | 0, Number(destinationX0) | 0, Number(destinationY0) | 0, Number(destinationX1) | 0, Number(destinationY1) | 0, Number(mask) >>> 0, Number(filter) >>> 0);
    }
    createVertexArray() {
        if (this.__contextLost) return null;
        return new WebGLVertexArrayObject(construct, this, call("webglCreateVertexArray", this.canvas));
    }
    bindVertexArray(array) {
        call("webglBindVertexArray", this.canvas, objectId(this, array, WebGLVertexArrayObject, true));
        this.__vertexArray = array;
    }
    deleteVertexArray(array) {
        if (array === null || array?.__id === 0) return;
        call("webglDeleteVertexArray", this.canvas, objectId(this, array, WebGLVertexArrayObject));
        if (this.__vertexArray === array) this.__vertexArray = null;
        array.__id = 0;
    }
    isVertexArray(array) {
        return array instanceof WebGLVertexArrayObject && array.__context === this && array.__id !== 0;
    }
    drawArraysInstanced(mode, first, count, instanceCount) {
        call("webglDrawArraysInstanced", this.canvas, Number(mode) >>> 0, Number(first) | 0, Number(count) | 0, Number(instanceCount) | 0);
    }
    drawElementsInstanced(mode, count, type, offset, instanceCount) {
        call("webglDrawElementsInstanced", this.canvas, Number(mode) >>> 0, Number(count) | 0, Number(type) >>> 0, Number(offset) | 0, Number(instanceCount) | 0);
    }
    vertexAttribDivisor(index, divisor) {
        call("webglVertexAttribDivisor", this.canvas, Number(index) >>> 0, Number(divisor) >>> 0);
    }
}

const constants = {
    DEPTH_BUFFER_BIT: 0x00000100, STENCIL_BUFFER_BIT: 0x00000400, COLOR_BUFFER_BIT: 0x00004000,
    NO_ERROR: 0, INVALID_ENUM: 0x0500, INVALID_VALUE: 0x0501,
    INVALID_OPERATION: 0x0502, OUT_OF_MEMORY: 0x0505,
    INVALID_FRAMEBUFFER_OPERATION: 0x0506, CONTEXT_LOST_WEBGL: 0x9242,
    VENDOR: 0x1F00, RENDERER: 0x1F01, VERSION: 0x1F02,
    SHADING_LANGUAGE_VERSION: 0x8B8C, ALPHA: 0x1906, RGB: 0x1907, RGBA: 0x1908,
    LUMINANCE: 0x1909, LUMINANCE_ALPHA: 0x190A,
    BYTE: 0x1400, UNSIGNED_BYTE: 0x1401, SHORT: 0x1402, UNSIGNED_SHORT: 0x1403,
    INT: 0x1404, UNSIGNED_INT: 0x1405,
    UNSIGNED_SHORT_4_4_4_4: 0x8033, UNSIGNED_SHORT_5_5_5_1: 0x8034,
    UNSIGNED_SHORT_5_6_5: 0x8363,
    VERTEX_SHADER: 0x8B31, FRAGMENT_SHADER: 0x8B30,
    DELETE_STATUS: 0x8B80, COMPILE_STATUS: 0x8B81, SHADER_TYPE: 0x8B4F,
    LINK_STATUS: 0x8B82, VALIDATE_STATUS: 0x8B83,
    ATTACHED_SHADERS: 0x8B85, ACTIVE_UNIFORMS: 0x8B86,
    ACTIVE_ATTRIBUTES: 0x8B89, CURRENT_PROGRAM: 0x8B8D,
    LOW_FLOAT: 0x8DF0, MEDIUM_FLOAT: 0x8DF1, HIGH_FLOAT: 0x8DF2,
    LOW_INT: 0x8DF3, MEDIUM_INT: 0x8DF4, HIGH_INT: 0x8DF5,
    ARRAY_BUFFER: 0x8892, ELEMENT_ARRAY_BUFFER: 0x8893,
    ARRAY_BUFFER_BINDING: 0x8894, ELEMENT_ARRAY_BUFFER_BINDING: 0x8895,
    BUFFER_SIZE: 0x8764, BUFFER_USAGE: 0x8765,
    CURRENT_VERTEX_ATTRIB: 0x8626,
    VERTEX_ATTRIB_ARRAY_ENABLED: 0x8622, VERTEX_ATTRIB_ARRAY_SIZE: 0x8623,
    VERTEX_ATTRIB_ARRAY_STRIDE: 0x8624, VERTEX_ATTRIB_ARRAY_TYPE: 0x8625,
    VERTEX_ATTRIB_ARRAY_NORMALIZED: 0x886A, VERTEX_ATTRIB_ARRAY_POINTER: 0x8645,
    VERTEX_ATTRIB_ARRAY_BUFFER_BINDING: 0x889F,
    STREAM_DRAW: 0x88E0, STATIC_DRAW: 0x88E4, DYNAMIC_DRAW: 0x88E8,
    DYNAMIC_COPY: 0x88EA, FLOAT: 0x1406,
    FLOAT_VEC2: 0x8B50, FLOAT_VEC3: 0x8B51, FLOAT_VEC4: 0x8B52,
    INT_VEC2: 0x8B53, INT_VEC3: 0x8B54, INT_VEC4: 0x8B55,
    BOOL: 0x8B56, BOOL_VEC2: 0x8B57, BOOL_VEC3: 0x8B58, BOOL_VEC4: 0x8B59,
    FLOAT_MAT2: 0x8B5A, FLOAT_MAT3: 0x8B5B, FLOAT_MAT4: 0x8B5C,
    SAMPLER_2D: 0x8B5E, SAMPLER_CUBE: 0x8B60,
    POINTS: 0x0000, LINES: 0x0001, LINE_LOOP: 0x0002, LINE_STRIP: 0x0003,
    TRIANGLES: 0x0004, TRIANGLE_STRIP: 0x0005, TRIANGLE_FAN: 0x0006,
    BLEND: 0x0BE2, CULL_FACE: 0x0B44, DEPTH_TEST: 0x0B71, DITHER: 0x0BD0,
    POLYGON_OFFSET_FILL: 0x8037, SAMPLE_ALPHA_TO_COVERAGE: 0x809E,
    SAMPLE_COVERAGE: 0x80A0, SCISSOR_TEST: 0x0C11, STENCIL_TEST: 0x0B90,
    LINE_WIDTH: 0x0B21, ALIASED_POINT_SIZE_RANGE: 0x846D,
    ALIASED_LINE_WIDTH_RANGE: 0x846E, CULL_FACE_MODE: 0x0B45,
    FRONT_FACE: 0x0B46, DEPTH_RANGE: 0x0B70, DEPTH_WRITEMASK: 0x0B72,
    DEPTH_CLEAR_VALUE: 0x0B73, DEPTH_FUNC: 0x0B74,
    STENCIL_CLEAR_VALUE: 0x0B91, STENCIL_FUNC: 0x0B92,
    STENCIL_VALUE_MASK: 0x0B93, STENCIL_FAIL: 0x0B94,
    STENCIL_PASS_DEPTH_FAIL: 0x0B95, STENCIL_PASS_DEPTH_PASS: 0x0B96,
    STENCIL_REF: 0x0B97, STENCIL_WRITEMASK: 0x0B98,
    VIEWPORT: 0x0BA2, SCISSOR_BOX: 0x0C10,
    SUBPIXEL_BITS: 0x0D50, RED_BITS: 0x0D52, GREEN_BITS: 0x0D53,
    BLUE_BITS: 0x0D54, ALPHA_BITS: 0x0D55, DEPTH_BITS: 0x0D56,
    STENCIL_BITS: 0x0D57,
    COLOR_CLEAR_VALUE: 0x0C22, COLOR_WRITEMASK: 0x0C23,
    PACK_ALIGNMENT: 0x0D05, POLYGON_OFFSET_UNITS: 0x2A00,
    NEVER: 0x0200, LESS: 0x0201, EQUAL: 0x0202, LEQUAL: 0x0203,
    GREATER: 0x0204, NOTEQUAL: 0x0205, GEQUAL: 0x0206, ALWAYS: 0x0207,
    ZERO: 0, ONE: 1, SRC_COLOR: 0x0300, ONE_MINUS_SRC_COLOR: 0x0301,
    SRC_ALPHA: 0x0302, ONE_MINUS_SRC_ALPHA: 0x0303, DST_ALPHA: 0x0304,
    ONE_MINUS_DST_ALPHA: 0x0305, DST_COLOR: 0x0306, ONE_MINUS_DST_COLOR: 0x0307,
    SRC_ALPHA_SATURATE: 0x0308, CONSTANT_COLOR: 0x8001,
    ONE_MINUS_CONSTANT_COLOR: 0x8002, CONSTANT_ALPHA: 0x8003,
    ONE_MINUS_CONSTANT_ALPHA: 0x8004, FUNC_ADD: 0x8006, BLEND_EQUATION: 0x8009,
    BLEND_COLOR: 0x8005, BLEND_DST_RGB: 0x80C8, BLEND_SRC_RGB: 0x80C9,
    BLEND_DST_ALPHA: 0x80CA, BLEND_SRC_ALPHA: 0x80CB,
    BLEND_EQUATION_RGB: 0x8009, BLEND_EQUATION_ALPHA: 0x883D,
    FUNC_SUBTRACT: 0x800A, FUNC_REVERSE_SUBTRACT: 0x800B,
    KEEP: 0x1E00, REPLACE: 0x1E01, INCR: 0x1E02, DECR: 0x1E03,
    INVERT: 0x150A, INCR_WRAP: 0x8507, DECR_WRAP: 0x8508,
    POLYGON_OFFSET_FACTOR: 0x8038, SAMPLE_BUFFERS: 0x80A8, SAMPLES: 0x80A9,
    SAMPLE_COVERAGE_VALUE: 0x80AA,
    SAMPLE_COVERAGE_INVERT: 0x80AB, STENCIL_BACK_FUNC: 0x8800,
    STENCIL_BACK_FAIL: 0x8801, STENCIL_BACK_PASS_DEPTH_FAIL: 0x8802,
    STENCIL_BACK_PASS_DEPTH_PASS: 0x8803, STENCIL_BACK_REF: 0x8CA3,
    STENCIL_BACK_VALUE_MASK: 0x8CA4, STENCIL_BACK_WRITEMASK: 0x8CA5,
    FRONT: 0x0404, BACK: 0x0405, FRONT_AND_BACK: 0x0408, CW: 0x0900, CCW: 0x0901,
    DONT_CARE: 0x1100, FASTEST: 0x1101, NICEST: 0x1102,
    GENERATE_MIPMAP_HINT: 0x8192,
    MAX_TEXTURE_SIZE: 0x0D33, MAX_VIEWPORT_DIMS: 0x0D3A, MAX_RENDERBUFFER_SIZE: 0x84E8,
    MAX_VERTEX_ATTRIBS: 0x8869, MAX_TEXTURE_IMAGE_UNITS: 0x8872,
    MAX_VERTEX_UNIFORM_VECTORS: 0x8DFB, MAX_VARYING_VECTORS: 0x8DFC,
    MAX_FRAGMENT_UNIFORM_VECTORS: 0x8DFD,
    MAX_COMBINED_TEXTURE_IMAGE_UNITS: 0x8B4D,
    MAX_VERTEX_TEXTURE_IMAGE_UNITS: 0x8B4C,
    COMPRESSED_TEXTURE_FORMATS: 0x86A3,
    TEXTURE_2D: 0x0DE1, TEXTURE_CUBE_MAP: 0x8513,
    TEXTURE_3D: 0x806F, TEXTURE_2D_ARRAY: 0x8C1A,
    TEXTURE_CUBE_MAP_POSITIVE_X: 0x8515, TEXTURE_CUBE_MAP_NEGATIVE_X: 0x8516,
    TEXTURE_CUBE_MAP_POSITIVE_Y: 0x8517, TEXTURE_CUBE_MAP_NEGATIVE_Y: 0x8518,
    TEXTURE_CUBE_MAP_POSITIVE_Z: 0x8519, TEXTURE_CUBE_MAP_NEGATIVE_Z: 0x851A,
    ACTIVE_TEXTURE: 0x84E0, TEXTURE_BINDING_2D: 0x8069,
    TEXTURE_BINDING_CUBE_MAP: 0x8514, MAX_CUBE_MAP_TEXTURE_SIZE: 0x851C,
    TEXTURE_BINDING_3D: 0x806A, TEXTURE_BINDING_2D_ARRAY: 0x8C1D,
    TEXTURE_MAG_FILTER: 0x2800, TEXTURE_MIN_FILTER: 0x2801,
    TEXTURE_WRAP_S: 0x2802, TEXTURE_WRAP_T: 0x2803,
    NEAREST: 0x2600, LINEAR: 0x2601, NEAREST_MIPMAP_NEAREST: 0x2700,
    LINEAR_MIPMAP_NEAREST: 0x2701, NEAREST_MIPMAP_LINEAR: 0x2702,
    LINEAR_MIPMAP_LINEAR: 0x2703, REPEAT: 0x2901, CLAMP_TO_EDGE: 0x812F,
    MIRRORED_REPEAT: 0x8370, UNPACK_ALIGNMENT: 0x0CF5,
    UNPACK_FLIP_Y_WEBGL: 0x9240, UNPACK_PREMULTIPLY_ALPHA_WEBGL: 0x9241,
    UNPACK_COLORSPACE_CONVERSION_WEBGL: 0x9243, BROWSER_DEFAULT_WEBGL: 0x9244,
    FRAMEBUFFER: 0x8D40, RENDERBUFFER: 0x8D41,
    TEXTURE: 0x1702, NONE: 0,
    READ_FRAMEBUFFER: 0x8CA8, DRAW_FRAMEBUFFER: 0x8CA9,
    FRAMEBUFFER_BINDING: 0x8CA6, RENDERBUFFER_BINDING: 0x8CA7,
    DRAW_FRAMEBUFFER_BINDING: 0x8CA6, READ_FRAMEBUFFER_BINDING: 0x8CAA,
    COLOR_ATTACHMENT0: 0x8CE0, DEPTH_ATTACHMENT: 0x8D00,
    STENCIL_ATTACHMENT: 0x8D20, DEPTH_STENCIL_ATTACHMENT: 0x821A,
    FRAMEBUFFER_COMPLETE: 0x8CD5, FRAMEBUFFER_INCOMPLETE_ATTACHMENT: 0x8CD6,
    FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: 0x8CD7,
    FRAMEBUFFER_INCOMPLETE_DIMENSIONS: 0x8CD9, FRAMEBUFFER_UNSUPPORTED: 0x8CDD,
    DEPTH_COMPONENT: 0x1902, DEPTH_COMPONENT16: 0x81A5,
    STENCIL_INDEX8: 0x8D48, COLOR: 0x1800, DEPTH: 0x1801, STENCIL: 0x1802,
    RENDERBUFFER_WIDTH: 0x8D42, RENDERBUFFER_HEIGHT: 0x8D43,
    RENDERBUFFER_INTERNAL_FORMAT: 0x8D44, RENDERBUFFER_RED_SIZE: 0x8D50,
    RENDERBUFFER_GREEN_SIZE: 0x8D51, RENDERBUFFER_BLUE_SIZE: 0x8D52,
    RENDERBUFFER_ALPHA_SIZE: 0x8D53, RENDERBUFFER_DEPTH_SIZE: 0x8D54,
    RENDERBUFFER_STENCIL_SIZE: 0x8D55,
    FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE: 0x8CD0,
    FRAMEBUFFER_ATTACHMENT_OBJECT_NAME: 0x8CD1,
    FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL: 0x8CD2,
    FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE: 0x8CD3,
    DEPTH_STENCIL: 0x84F9,
    RGBA4: 0x8056, RGB5_A1: 0x8057, RGBA8: 0x8058, RGB565: 0x8D62,
    MAX_SAMPLES: 0x8D57,
    IMPLEMENTATION_COLOR_READ_TYPE: 0x8B9A, IMPLEMENTATION_COLOR_READ_FORMAT: 0x8B9B,
    UNMASKED_VENDOR_WEBGL: 0x9245, UNMASKED_RENDERER_WEBGL: 0x9246,
};
const webgl2Constants = {
    RGB8: 0x8051, RGB10_A2: 0x8059,
    MAX_3D_TEXTURE_SIZE: 0x8073, MAX_ELEMENTS_VERTICES: 0x80E8,
    MAX_ELEMENTS_INDICES: 0x80E9, MIN: 0x8007, MAX: 0x8008,
    MAX_TEXTURE_LOD_BIAS: 0x84FD,
    MAX_FRAGMENT_UNIFORM_COMPONENTS: 0x8B49,
    MAX_VERTEX_UNIFORM_COMPONENTS: 0x8B4A,
    SAMPLER_3D: 0x8B5F, SAMPLER_2D_SHADOW: 0x8B62,
    FRAGMENT_SHADER_DERIVATIVE_HINT: 0x8B8B,
    SRGB: 0x8C40, SRGB8: 0x8C41, SRGB8_ALPHA8: 0x8C43,
    MAX_ARRAY_TEXTURE_LAYERS: 0x88FF, MIN_PROGRAM_TEXEL_OFFSET: 0x8904,
    MAX_PROGRAM_TEXEL_OFFSET: 0x8905, MAX_VARYING_COMPONENTS: 0x8B4B,
    R11F_G11F_B10F: 0x8C3A, RGB9_E5: 0x8C3D,
    MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS: 0x8C80,
    MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS: 0x8C8A,
    MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS: 0x8C8B,
    RGBA32UI: 0x8D70, RGB32UI: 0x8D71,
    RGBA16UI: 0x8D76, RGB16UI: 0x8D77, RGB8UI: 0x8D7D,
    RGBA32I: 0x8D82, RGB32I: 0x8D83,
    RGBA16I: 0x8D88, RGB16I: 0x8D89, RGB8I: 0x8D8F,
    SAMPLER_2D_ARRAY: 0x8DC1, SAMPLER_2D_ARRAY_SHADOW: 0x8DC4,
    SAMPLER_CUBE_SHADOW: 0x8DC5,
    INT_SAMPLER_2D: 0x8DCA, INT_SAMPLER_3D: 0x8DCB,
    INT_SAMPLER_CUBE: 0x8DCC, INT_SAMPLER_2D_ARRAY: 0x8DCF,
    UNSIGNED_INT_SAMPLER_2D: 0x8DD2, UNSIGNED_INT_SAMPLER_3D: 0x8DD3,
    UNSIGNED_INT_SAMPLER_CUBE: 0x8DD4, UNSIGNED_INT_SAMPLER_2D_ARRAY: 0x8DD7,
    FRAMEBUFFER_DEFAULT: 0x8218, UNSIGNED_NORMALIZED: 0x8C17,
    FRAMEBUFFER_INCOMPLETE_MULTISAMPLE: 0x8D56,
    R8: 0x8229, RG8: 0x822B,
    R8I: 0x8231, R8UI: 0x8232, R16I: 0x8233, R16UI: 0x8234,
    R32I: 0x8235, R32UI: 0x8236,
    RG8I: 0x8237, RG8UI: 0x8238, RG16I: 0x8239, RG16UI: 0x823A,
    RG32I: 0x823B, RG32UI: 0x823C,
    VERTEX_ARRAY_BINDING: 0x85B5,
    R8_SNORM: 0x8F94, RG8_SNORM: 0x8F95,
    RGB8_SNORM: 0x8F96, RGBA8_SNORM: 0x8F97,
    SIGNED_NORMALIZED: 0x8F9C,
    MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS: 0x8A31,
    MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS: 0x8A33,
    MAX_VERTEX_OUTPUT_COMPONENTS: 0x9122, MAX_FRAGMENT_INPUT_COMPONENTS: 0x9125,
    MAX_SERVER_WAIT_TIMEOUT: 0x9111, OBJECT_TYPE: 0x9112, SYNC_FENCE: 0x9116,
    RGB10_A2UI: 0x906F, INT_2_10_10_10_REV: 0x8D9F,
    TEXTURE_IMMUTABLE_FORMAT: 0x912F, TEXTURE_IMMUTABLE_LEVELS: 0x82DF,
    MAX_ELEMENT_INDEX: 0x8D6B,
    MAX_DRAW_BUFFERS: 0x8824, MAX_COLOR_ATTACHMENTS: 0x8CDF,
    READ_BUFFER: 0x0C02, UNPACK_ROW_LENGTH: 0x0CF2,
    UNPACK_SKIP_ROWS: 0x0CF3, UNPACK_SKIP_PIXELS: 0x0CF4,
    PACK_ROW_LENGTH: 0x0D02, PACK_SKIP_ROWS: 0x0D03, PACK_SKIP_PIXELS: 0x0D04,
    UNPACK_SKIP_IMAGES: 0x806D, UNPACK_IMAGE_HEIGHT: 0x806E,
    COPY_READ_BUFFER: 0x8F36, COPY_WRITE_BUFFER: 0x8F37,
    COPY_READ_BUFFER_BINDING: 0x8F36, COPY_WRITE_BUFFER_BINDING: 0x8F37,
    STREAM_READ: 0x88E1, STREAM_COPY: 0x88E2,
    STATIC_READ: 0x88E5, STATIC_COPY: 0x88E6, DYNAMIC_READ: 0x88E9,
    PIXEL_PACK_BUFFER: 0x88EB, PIXEL_UNPACK_BUFFER: 0x88EC,
    PIXEL_PACK_BUFFER_BINDING: 0x88ED, PIXEL_UNPACK_BUFFER_BINDING: 0x88EF,
    TEXTURE_BASE_LEVEL: 0x813C, TEXTURE_MAX_LEVEL: 0x813D,
    VERTEX_ATTRIB_ARRAY_INTEGER: 0x88FD, VERTEX_ATTRIB_ARRAY_DIVISOR: 0x88FE,
    RENDERBUFFER_SAMPLES: 0x8CAB,
    FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER: 0x8CD4,
    FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING: 0x8210,
    FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE: 0x8211,
    FRAMEBUFFER_ATTACHMENT_RED_SIZE: 0x8212,
    FRAMEBUFFER_ATTACHMENT_GREEN_SIZE: 0x8213,
    FRAMEBUFFER_ATTACHMENT_BLUE_SIZE: 0x8214,
    FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE: 0x8215,
    FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE: 0x8216,
    FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE: 0x8217,
    RED: 0x1903, RG: 0x8227, RED_INTEGER: 0x8D94, RG_INTEGER: 0x8228,
    RGB_INTEGER: 0x8D98, RGBA_INTEGER: 0x8D99,
    HALF_FLOAT: 0x140B, UNSIGNED_INT_2_10_10_10_REV: 0x8368,
    UNSIGNED_INT_10F_11F_11F_REV: 0x8C3B, UNSIGNED_INT_5_9_9_9_REV: 0x8C3E,
    UNSIGNED_INT_24_8: 0x84FA,
    FLOAT_32_UNSIGNED_INT_24_8_REV: 0x8DAD,
    R16F: 0x822D, RG16F: 0x822F, RGB16F: 0x881B, RGBA16F: 0x881A,
    R32F: 0x822E, RG32F: 0x8230, RGB32F: 0x8815, RGBA32F: 0x8814,
    RGBA8UI: 0x8D7C, RGBA8I: 0x8D8E,
    DEPTH_COMPONENT24: 0x81A6, DEPTH_COMPONENT32F: 0x8CAC,
    DEPTH24_STENCIL8: 0x88F0, DEPTH32F_STENCIL8: 0x8CAD,
    SAMPLER_BINDING: 0x8919, TEXTURE_WRAP_R: 0x8072,
    TEXTURE_MIN_LOD: 0x813A, TEXTURE_MAX_LOD: 0x813B,
    TEXTURE_COMPARE_MODE: 0x884C, TEXTURE_COMPARE_FUNC: 0x884D,
    COMPARE_REF_TO_TEXTURE: 0x884E,
    ANY_SAMPLES_PASSED: 0x8C2F, ANY_SAMPLES_PASSED_CONSERVATIVE: 0x8D6A,
    TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN: 0x8C88,
    CURRENT_QUERY: 0x8865, QUERY_RESULT: 0x8866, QUERY_RESULT_AVAILABLE: 0x8867,
    SYNC_CONDITION: 0x9113, SYNC_STATUS: 0x9114, SYNC_FLAGS: 0x9115,
    SYNC_GPU_COMMANDS_COMPLETE: 0x9117, UNSIGNALED: 0x9118, SIGNALED: 0x9119,
    ALREADY_SIGNALED: 0x911A, TIMEOUT_EXPIRED: 0x911B,
    CONDITION_SATISFIED: 0x911C, WAIT_FAILED: 0x911D,
    SYNC_FLUSH_COMMANDS_BIT: 0x00000001, TIMEOUT_IGNORED: -1,
    MAX_CLIENT_WAIT_TIMEOUT_WEBGL: 0x9247,
    TRANSFORM_FEEDBACK: 0x8E22, TRANSFORM_FEEDBACK_PAUSED: 0x8E23,
    TRANSFORM_FEEDBACK_ACTIVE: 0x8E24, TRANSFORM_FEEDBACK_BINDING: 0x8E25,
    TRANSFORM_FEEDBACK_BUFFER: 0x8C8E,
    TRANSFORM_FEEDBACK_BUFFER_BINDING: 0x8C8F,
    TRANSFORM_FEEDBACK_BUFFER_START: 0x8C84, TRANSFORM_FEEDBACK_BUFFER_SIZE: 0x8C85,
    INTERLEAVED_ATTRIBS: 0x8C8C, SEPARATE_ATTRIBS: 0x8C8D,
    TRANSFORM_FEEDBACK_BUFFER_MODE: 0x8C7F, TRANSFORM_FEEDBACK_VARYINGS: 0x8C83,
    RASTERIZER_DISCARD: 0x8C89,
    FLOAT_MAT2x3: 0x8B65, FLOAT_MAT2x4: 0x8B66, FLOAT_MAT3x2: 0x8B67,
    FLOAT_MAT3x4: 0x8B68, FLOAT_MAT4x2: 0x8B69, FLOAT_MAT4x3: 0x8B6A,
    UNSIGNED_INT_VEC2: 0x8DC6, UNSIGNED_INT_VEC3: 0x8DC7,
    UNSIGNED_INT_VEC4: 0x8DC8,
    UNIFORM_BUFFER: 0x8A11, UNIFORM_BUFFER_BINDING: 0x8A28,
    UNIFORM_BUFFER_START: 0x8A29, UNIFORM_BUFFER_SIZE: 0x8A2A,
    MAX_VERTEX_UNIFORM_BLOCKS: 0x8A2B, MAX_FRAGMENT_UNIFORM_BLOCKS: 0x8A2D,
    MAX_COMBINED_UNIFORM_BLOCKS: 0x8A2E, MAX_UNIFORM_BUFFER_BINDINGS: 0x8A2F,
    MAX_UNIFORM_BLOCK_SIZE: 0x8A30, UNIFORM_BUFFER_OFFSET_ALIGNMENT: 0x8A34,
    ACTIVE_UNIFORM_BLOCKS: 0x8A36, UNIFORM_TYPE: 0x8A37, UNIFORM_SIZE: 0x8A38,
    UNIFORM_BLOCK_INDEX: 0x8A3A, UNIFORM_OFFSET: 0x8A3B,
    UNIFORM_ARRAY_STRIDE: 0x8A3C, UNIFORM_MATRIX_STRIDE: 0x8A3D,
    UNIFORM_IS_ROW_MAJOR: 0x8A3E, UNIFORM_BLOCK_BINDING: 0x8A3F,
    UNIFORM_BLOCK_DATA_SIZE: 0x8A40, UNIFORM_BLOCK_ACTIVE_UNIFORMS: 0x8A42,
    UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES: 0x8A43,
    UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER: 0x8A44,
    UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER: 0x8A46,
    INVALID_INDEX: 0xFFFFFFFF,
};
for (let index = 0; index < 32; index += 1) constants[`TEXTURE${index}`] = 0x84C0 + index;
for (let index = 0; index < 16; index += 1) {
    webgl2Constants[`DRAW_BUFFER${index}`] = 0x8825 + index;
    webgl2Constants[`COLOR_ATTACHMENT${index}`] = 0x8CE0 + index;
}
for (const constructor of [WebGLRenderingContext, WebGL2RenderingContext]) {
    for (const [name, value] of Object.entries(constants)) {
        Object.defineProperty(constructor, name, { value, enumerable: true });
        Object.defineProperty(constructor.prototype, name, { value, enumerable: true });
    }
}
for (const [name, value] of Object.entries(webgl2Constants)) {
    Object.defineProperty(WebGL2RenderingContext, name, { value, enumerable: true });
    Object.defineProperty(WebGL2RenderingContext.prototype, name, { value, enumerable: true });
}

Object.defineProperties(globalThis, {
    WebGLRenderingContext: { value: WebGLRenderingContext, writable: true, configurable: true },
    WebGL2RenderingContext: { value: WebGL2RenderingContext, writable: true, configurable: true },
    WebGLShader: { value: WebGLShader, writable: true, configurable: true },
    WebGLProgram: { value: WebGLProgram, writable: true, configurable: true },
    WebGLBuffer: { value: WebGLBuffer, writable: true, configurable: true },
    WebGLTexture: { value: WebGLTexture, writable: true, configurable: true },
    WebGLFramebuffer: { value: WebGLFramebuffer, writable: true, configurable: true },
    WebGLRenderbuffer: { value: WebGLRenderbuffer, writable: true, configurable: true },
    WebGLSampler: { value: WebGLSampler, writable: true, configurable: true },
    WebGLQuery: { value: WebGLQuery, writable: true, configurable: true },
    WebGLSync: { value: WebGLSync, writable: true, configurable: true },
    WebGLTransformFeedback: { value: WebGLTransformFeedback, writable: true, configurable: true },
    WebGLActiveInfo: { value: WebGLActiveInfo, writable: true, configurable: true },
    WebGLShaderPrecisionFormat: { value: WebGLShaderPrecisionFormat, writable: true, configurable: true },
    WebGLUniformLocation: { value: WebGLUniformLocation, writable: true, configurable: true },
    WebGLVertexArrayObject: { value: WebGLVertexArrayObject, writable: true, configurable: true },
    WebGLContextEvent: { value: WebGLContextEvent, writable: true, configurable: true },
    __brimpSetWebGlPersona: {
        value(value) { graphicsPersona = Object.freeze({ ...value }); },
        configurable: true,
    },
    __brimpCreateWebGLContext: {
        value(canvas, type) {
            const version = type === "webgl2" ? 2 : 1;
            if (!call("webglAcquire", canvas, canvas.width, canvas.height, version)) return null;
            const context = version === 2 ? new WebGL2RenderingContext(construct, canvas) : new WebGLRenderingContext(construct, canvas, 1);
            contextsByCanvas.set(canvas, context);
            return context;
        },
        configurable: true,
    },
});

Object.defineProperties(HTMLCanvasElement.prototype, {
    onwebglcontextlost: { value: null, writable: true, configurable: true, enumerable: true },
    onwebglcontextrestored: { value: null, writable: true, configurable: true, enumerable: true },
});

for (const constructor of [WebGLRenderingContext, WebGL2RenderingContext, WebGLShader, WebGLProgram, WebGLBuffer, WebGLTexture, WebGLFramebuffer, WebGLRenderbuffer, WebGLSampler, WebGLQuery, WebGLTimerQueryEXT, WebGLSync, WebGLTransformFeedback, WebGLActiveInfo, WebGLShaderPrecisionFormat, WebGLUniformLocation, WebGLVertexArrayObject, WebGLContextEvent, WebGLLoseContext, WEBGLDebugShaders, EXTClipControl, EXTPolygonOffsetClamp, WEBGLProvokingVertex, WEBGLPolygonMode, OESVertexArrayObject, ANGLEInstancedArrays, WEBGLDrawBuffers, WEBGLCompressedTextureASTC, EXTDisjointTimerQuery, EXTDisjointTimerQueryWebGL2, OESDrawBuffersIndexed, WEBGLMultiDraw, WEBGLDrawInstancedBaseVertexBaseInstance, WEBGLMultiDrawInstancedBaseVertexBaseInstance]) {
    globalThis.__brimpMarkWebBuiltin?.(constructor);
    for (const key of Reflect.ownKeys(constructor.prototype)) {
        if (key === "constructor") continue;
        const descriptor = Object.getOwnPropertyDescriptor(constructor.prototype, key);
        if (typeof descriptor?.value === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.value, `function ${String(key)}() { [native code] }`);
        if (typeof descriptor?.get === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.get, `function get ${String(key)}() { [native code] }`);
    }
}
})();
