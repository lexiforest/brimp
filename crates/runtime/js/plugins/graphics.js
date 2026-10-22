({global,host}, options={}) => {
    const rgba=Array.from(options.rgba??[0,0,0,0],Number);
    if(rgba.length!==4||rgba.some(n=>!Number.isInteger(n)||n<0||n>255))throw new TypeError('rgba requires four bytes');
    const constants={NO_ERROR:0,INVALID_ENUM:0x500,INVALID_VALUE:0x501,INVALID_OPERATION:0x502,RGBA:0x1908,UNSIGNED_BYTE:0x1401,ARRAY_BUFFER:0x8892,STATIC_DRAW:0x88E4,FLOAT:0x1406,TRIANGLES:4,COLOR_BUFFER_BIT:0x4000,DEPTH_BUFFER_BIT:0x100,VERTEX_SHADER:0x8B31,FRAGMENT_SHADER:0x8B30,COMPILE_STATUS:0x8B81,LINK_STATUS:0x8B82,VERSION:0x1F02,VENDOR:0x1F00,RENDERER:0x1F01,MAX_TEXTURE_SIZE:0x0D33,VIEWPORT:0x0BA2,COLOR_CLEAR_VALUE:0x0C22};
    const contexts=new WeakMap(),state=new WeakMap(),key={};
    function dimensions(canvas){return [Number(canvas.width),Number(canvas.height)];}
    class WebGLRenderingContext {
        constructor(token,canvas,attributes={}){if(token!==key)throw new TypeError('Illegal constructor');state.set(this,{canvas,attributes:{alpha:true,...attributes},error:0,clear:[0,0,0,0],viewport:[0,0,...dimensions(canvas)],buffers:new Map(),bound:null});}
        get canvas(){return state.get(this).canvas;}get drawingBufferWidth(){return this.canvas.width;}get drawingBufferHeight(){return this.canvas.height;}
        getContextAttributes(){return {...state.get(this).attributes};}isContextLost(){return false;}
        getSupportedExtensions(){return []; }getExtension(){return null;}
        getError(){const s=state.get(this),error=s.error;s.error=0;return error;}
        getParameter(name){const s=state.get(this);if(name===this.VERSION)return this instanceof WebGL2RenderingContext?'WebGL 2.0 (Brimp deterministic mock)':'WebGL 1.0 (Brimp deterministic mock)';if(name===this.VENDOR)return 'Brimp';if(name===this.RENDERER)return String(options.renderer??'Deterministic JavaScript mock');if(name===this.MAX_TEXTURE_SIZE)return 4096;if(name===this.VIEWPORT)return new Int32Array(s.viewport);if(name===this.COLOR_CLEAR_VALUE)return new Float32Array(s.clear);s.error=this.INVALID_ENUM;return null;}
        clearColor(r,g,b,a){state.get(this).clear=[r,g,b,a].map(v=>Math.max(0,Math.min(1,Number(v))));}
        clear(){}flush(){}finish(){}
        viewport(x,y,width,height){if(width<0||height<0){state.get(this).error=this.INVALID_VALUE;return;}state.get(this).viewport=[x,y,width,height].map(Math.trunc);}
        readPixels(x,y,width,height,format,type,pixels){const s=state.get(this);if(format!==this.RGBA||type!==this.UNSIGNED_BYTE){s.error=this.INVALID_ENUM;return;}if(![x,y,width,height].every(Number.isInteger)||width<0||height<0||width*height>16_000_000||!(pixels instanceof Uint8Array)||pixels.length<width*height*4){s.error=this.INVALID_VALUE;return;}for(let row=0;row<height;row++)for(let col=0;col<width;col++)for(let c=0;c<4;c++)pixels[(row*width+col)*4+c]=(x+col<0||y+row<0||x+col>=this.drawingBufferWidth||y+row>=this.drawingBufferHeight)?0:(!s.attributes.alpha&&c===3?255:rgba[c]);}
        createBuffer(){const buffer={};state.get(this).buffers.set(buffer,new Uint8Array());return buffer;}
        bindBuffer(target,buffer){const s=state.get(this);if(target!==this.ARRAY_BUFFER){s.error=this.INVALID_ENUM;return;}if(buffer!==null&&!s.buffers.has(buffer)){s.error=this.INVALID_OPERATION;return;}s.bound=buffer;}
        bufferData(target,data){const s=state.get(this);if(target!==this.ARRAY_BUFFER||!s.bound){s.error=this.INVALID_OPERATION;return;}const length=typeof data==='number'?data:data?.byteLength;if(!Number.isInteger(length)||length<0||length>64*1024*1024){s.error=this.INVALID_VALUE;return;}s.buffers.set(s.bound,typeof data==='number'?new Uint8Array(length):new Uint8Array(data.buffer??data,data.byteOffset??0,length).slice());}
        deleteBuffer(buffer){const s=state.get(this);s.buffers.delete(buffer);if(s.bound===buffer)s.bound=null;}
        createShader(type){if(type!==this.VERTEX_SHADER&&type!==this.FRAGMENT_SHADER){state.get(this).error=this.INVALID_ENUM;return null;}return {type,source:''};}
        shaderSource(shader,source){shader.source=String(source);}compileShader(){}getShaderParameter(_,name){return name===this.COMPILE_STATUS?false:null;}getShaderInfoLog(){return 'Shader compilation unavailable in the Brimp mock';}
        createProgram(){return {};}attachShader(){}linkProgram(){}getProgramParameter(_,name){return name===this.LINK_STATUS?false:null;}getProgramInfoLog(){return 'Program linking unavailable in the Brimp mock';}
        drawArrays(){state.get(this).error=this.INVALID_OPERATION;}
        submitAsync(commands){return host.call('graphics.submit',{width:this.drawingBufferWidth,height:this.drawingBufferHeight,commands});}
    }
    class WebGL2RenderingContext extends WebGLRenderingContext {}
    Object.assign(WebGLRenderingContext.prototype,constants);Object.assign(WebGLRenderingContext,constants);Object.assign(WebGL2RenderingContext,constants);
    const original=global.HTMLCanvasElement.prototype.getContext;
    Object.defineProperty(global.HTMLCanvasElement.prototype,'getContext',{configurable:true,writable:true,value:function(type,attributes){type=String(type);if(contexts.has(this)){const saved=contexts.get(this);return saved.type===type?saved.context:null;}if(type==='webgl'||type==='webgl2'){const context=new (type==='webgl2'?WebGL2RenderingContext:WebGLRenderingContext)(key,this,attributes);contexts.set(this,{type,context});return context;}const context=original?.call(this,type,attributes)??null;if(context)contexts.set(this,{type,context});return context;}});
    const bufferState=new WeakMap();
    class GPUBuffer {
        constructor(token,descriptor){if(token!==key)throw new TypeError('Illegal constructor');const size=Number(descriptor.size);if(!Number.isSafeInteger(size)||size<0||size>64*1024*1024)throw new RangeError('GPU buffer size limit');this.size=size;this.usage=descriptor.usage??0;this.label=descriptor.label??'';bufferState.set(this,{bytes:new ArrayBuffer(size),mapped:!!descriptor.mappedAtCreation,destroyed:false});}
        get mapState(){return bufferState.get(this).mapped?'mapped':'unmapped';}
        async mapAsync(){const s=bufferState.get(this);if(s.destroyed||s.mapped)throw new DOMException('Invalid map state','OperationError');s.mapped=true;}
        getMappedRange(offset=0,size=this.size-offset){const s=bufferState.get(this);if(!s.mapped||s.destroyed)throw new DOMException('Buffer not mapped','OperationError');if(offset!==0||size!==this.size)throw new DOMException('Only full buffer mappings supported','NotSupportedError');return s.bytes;}
        unmap(){const s=bufferState.get(this);s.bytes=s.bytes.slice(0);s.mapped=false;}
        destroy(){const s=bufferState.get(this);s.destroyed=true;s.mapped=false;s.bytes=new ArrayBuffer(0);}
    }
    class GPUDevice extends EventTarget {
        constructor(){super();this.features=new Set();this.limits=Object.freeze({maxBufferSize:64*1024*1024});this.queue={submit:()=>{throw new DOMException('GPU execution unavailable','NotSupportedError');},onSubmittedWorkDone:()=>Promise.resolve(),writeBuffer:(buffer,offset,data)=>{const s=bufferState.get(buffer);if(!s||s.destroyed||s.mapped)throw new DOMException('Buffer unavailable','OperationError');const bytes=new Uint8Array(data.buffer??data,data.byteOffset??0,data.byteLength);if(!Number.isSafeInteger(offset)||offset<0||offset+bytes.length>buffer.size)throw new RangeError('Buffer range');new Uint8Array(s.bytes).set(bytes,offset);}};this.lost=new Promise(resolve=>{this._lose=resolve;});}
        createBuffer(descriptor){if(this._destroyed)throw new DOMException('Device destroyed','InvalidStateError');return new GPUBuffer(key,descriptor);}
        createShaderModule(){throw new DOMException('Shader compilation unavailable','NotSupportedError');}
        destroy(){this._destroyed=true;this._lose({reason:'destroyed',message:'Mock device destroyed'});}
        submitAsync(commands){return host.call('graphics.submit',commands);}
    }
    class GPUAdapter {constructor(){this.features=new Set();this.limits=Object.freeze({maxBufferSize:64*1024*1024});this.info=Object.freeze({vendor:'brimp',architecture:'mock',device:'mock',description:'Deterministic JavaScript mock'});}async requestDevice(){return new GPUDevice();}}
    class GPU {async requestAdapter(){return options.webgpu===true?new GPUAdapter():null;}getPreferredCanvasFormat(){return 'bgra8unorm';}}
    Object.defineProperty(global.navigator,'gpu',{value:new GPU(),configurable:true});
    for(const [name,value] of Object.entries({WebGLRenderingContext,WebGL2RenderingContext,GPU,GPUAdapter,GPUDevice,GPUBuffer,GPUBufferUsage:{MAP_READ:1,MAP_WRITE:2,COPY_SRC:4,COPY_DST:8,INDEX:16,VERTEX:32,UNIFORM:64,STORAGE:128,INDIRECT:256,QUERY_RESOLVE:512},GPUMapMode:{READ:1,WRITE:2}}))Object.defineProperty(global,name,{value,writable:true,configurable:true});
}
