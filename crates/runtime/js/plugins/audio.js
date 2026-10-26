({global,host}, options={}) => {
    const contexts=new Set();
    const construct=Symbol('audio interface construction');
    const validRate=value=>{value=Number(value);if(!Number.isFinite(value)||value<8000||value>96000)throw new DOMException('Unsupported sample rate','NotSupportedError');return value;};
    const validateBufferSize=(length,channels)=>{if(!Number.isInteger(length)||length<=0||!Number.isInteger(channels)||channels<1||channels>32||length*channels>16_000_000)throw new DOMException('Audio buffer size limit','NotSupportedError');};
    class AudioBuffer {
        #channels;
        constructor({length,numberOfChannels=1,sampleRate}){validateBufferSize(length,numberOfChannels);Object.defineProperties(this,{length:{value:length},sampleRate:{value:validRate(sampleRate)},numberOfChannels:{value:numberOfChannels}});this.#channels=Array.from({length:numberOfChannels},()=>new Float32Array(length));}
        get duration(){return this.length/this.sampleRate;}
        getChannelData(channel){if(!Number.isInteger(channel)||!this.#channels[channel])throw new DOMException('Invalid channel','IndexSizeError');return this.#channels[channel];}
        copyFromChannel(destination,channel,start=0){if(!(destination instanceof Float32Array))throw new TypeError('Float32Array required');if(!Number.isInteger(start)||start<0)throw new RangeError('Invalid offset');destination.set(this.getChannelData(channel).subarray(start,start+destination.length));}
        copyToChannel(source,channel,start=0){if(!(source instanceof Float32Array))throw new TypeError('Float32Array required');const destination=this.getChannelData(channel);if(!Number.isInteger(start)||start<0)throw new RangeError('Invalid offset');if(start<destination.length)destination.set(source.subarray(0,destination.length-start),start);}
    }
    class AudioParam {
        constructor(value){this.value=value;this.defaultValue=value;this.minValue=-3.4028234663852886e38;this.maxValue=3.4028234663852886e38;this.automationRate='a-rate';}
        setValueAtTime(value,time){if(!Number.isFinite(Number(value))||!Number.isFinite(Number(time))||time<0)throw new RangeError('Invalid audio parameter');this.value=Number(value);return this;}
        cancelScheduledValues(){return this;}cancelAndHoldAtTime(){return this;}
        // Legacy callable placeholders: automation does not schedule processing.
        linearRampToValueAtTime(){}
        exponentialRampToValueAtTime(){}
        setTargetAtTime(){}
        setValueCurveAtTime(){}
    }
    class AudioNode extends EventTarget {
        constructor(context){super();if(!(context instanceof BaseAudioContext))throw new TypeError('BaseAudioContext required');this.context=context;this.channelCount=2;this.channelCountMode='max';this.channelInterpretation='speakers';this.numberOfInputs=1;this.numberOfOutputs=1;this._connections=new Set();}
        connect(destination){if(this.context.state==='closed')throw new DOMException('Context closed','InvalidStateError');if(!(destination instanceof AudioNode)&&!(destination instanceof AudioParam))throw new TypeError('Invalid destination');if(destination.context&&destination.context!==this.context)throw new DOMException('Different context','InvalidAccessError');this._connections.add(destination);return destination instanceof AudioNode?destination:undefined;}
        disconnect(){this._connections.clear();}
    }
    class GainNode extends AudioNode {constructor(context,options={}){super(context);this.gain=new AudioParam(options.gain??1);}}
    class AudioScheduledSourceNode extends AudioNode {
        constructor(context){super(context);this.onended=null;}
        start(when=0){if(this._started)throw new DOMException('Already started','InvalidStateError');if(!Number.isFinite(Number(when))||when<0)throw new RangeError('Invalid start');this._started=true;}
        stop(when=0){if(!this._started)throw new DOMException('Not started','InvalidStateError');if(!Number.isFinite(Number(when))||when<0)throw new RangeError('Invalid stop');if(this._timer!==undefined)clearTimeout(this._timer);this._timer=setTimeout(()=>{this.dispatchEvent(new Event('ended'));if(typeof this.onended==='function')this.onended(new Event('ended'));},Math.max(0,(when-this.context.currentTime)*1000));this.context._timers.add(this._timer);}
    }
    class OscillatorNode extends AudioScheduledSourceNode {
        constructor(context,options={}){super(context);this.frequency=new AudioParam(options.frequency??440);this.detune=new AudioParam(options.detune??0);this.type=options.type??'sine';this.numberOfInputs=0;}
        setPeriodicWave(){}
    }
    class AudioDestinationNode extends AudioNode {
        constructor(context){super(context);this.numberOfOutputs=0;this.maxChannelCount=2;}
    }
    class AudioListener {
        constructor(token){
            if(token!==construct)throw new TypeError('Illegal constructor');
            for(const [name,value]of Object.entries({positionX:0,positionY:0,positionZ:0,forwardX:0,forwardY:0,forwardZ:-1,upX:0,upY:1,upZ:0}))this[name]=new AudioParam(value);
        }
        setPosition(x,y,z){this.positionX.value=Number(x);this.positionY.value=Number(y);this.positionZ.value=Number(z);}
        setOrientation(x,y,z,upX,upY,upZ){for(const [name,value]of Object.entries({forwardX:x,forwardY:y,forwardZ:z,upX,upY,upZ}))this[name].value=Number(value);}
    }
    class DynamicsCompressorNode extends AudioNode {
        constructor(context,options={}){
            super(context);this.channelCountMode='clamped-max';this.reduction=0;
            for(const [name,value,min,max]of [['attack',0.003,0,1],['knee',30,0,40],['ratio',12,1,20],['release',0.25,0,1],['threshold',-24,-100,0]]){
                const param=new AudioParam(value);param.value=options[name]??value;param.minValue=min;param.maxValue=max;param.automationRate='k-rate';this[name]=param;
            }
        }
    }
    class Worklet {
        constructor(token){if(token!==construct)throw new TypeError('Illegal constructor');}
        addModule(){}
    }
    class AudioWorklet extends Worklet {}
    class AnalyserNode extends AudioNode {
        #fftSize=2048;
        get fftSize(){return this.#fftSize;}set fftSize(v){v=Number(v);if(!Number.isInteger(v)||v<32||v>32768||(v&(v-1)))throw new DOMException('Invalid FFT size','IndexSizeError');this.#fftSize=v;}
        get frequencyBinCount(){return this.fftSize/2;}
        getFloatFrequencyData(array){array.fill(-Infinity,0,Math.min(array.length,this.frequencyBinCount));}
        getByteFrequencyData(array){array.fill(0,0,Math.min(array.length,this.frequencyBinCount));}
        getFloatTimeDomainData(array){array.fill(0,0,Math.min(array.length,this.fftSize));}
        getByteTimeDomainData(array){array.fill(128,0,Math.min(array.length,this.fftSize));}
    }
    class BaseAudioContext extends EventTarget {
        #state='suspended';#elapsed=0;#started=0;
        constructor(config={}){super();if(new.target===BaseAudioContext)throw new TypeError('Illegal constructor');this.sampleRate=validRate(config.sampleRate??options.sampleRate??48000);this.destination=new AudioDestinationNode(this);this.listener=new AudioListener(construct);this.audioWorklet=new AudioWorklet(construct);this.onstatechange=null;this._timers=new Set();}
        get state(){return this.#state;}get currentTime(){return this.#elapsed+(this.#state==='running'?(performance.now()-this.#started)/1000:0);}
        _changeState(state){if(this.#state==='closed')throw new DOMException('Context closed','InvalidStateError');this.#elapsed=this.currentTime;this.#started=performance.now();this.#state=state;this.dispatchEvent(new Event('statechange'));if(typeof this.onstatechange==='function')this.onstatechange(new Event('statechange'));}
        createBuffer(channels,length,sampleRate){return new AudioBuffer({numberOfChannels:channels,length,sampleRate});}
        createGain(){return new GainNode(this);}createOscillator(){return new OscillatorNode(this);}createAnalyser(){return new AnalyserNode(this);}
        createDynamicsCompressor(){return new DynamicsCompressorNode(this);}
        // The remaining legacy factories are callable without creating engines.
        createBiquadFilter(){}createBufferSource(){}createChannelMerger(){}createChannelSplitter(){}
        createConstantSource(){}createConvolver(){}createDelay(){}createIIRFilter(){}
        createPanner(){}createPeriodicWave(){}createScriptProcessor(){}createStereoPanner(){}createWaveShaper(){}
        async decodeAudioData(){throw new DOMException('Audio decoding unavailable','NotSupportedError');}
    }
    class AudioContext extends BaseAudioContext {
        constructor(config={}){super(config);contexts.add(this);}
        async resume(){this._changeState('running');}async suspend(){this._changeState('suspended');}
        async close(){this._changeState('closed');for(const timer of this._timers)clearTimeout(timer);this._timers.clear();contexts.delete(this);}
        processAsync(samples){return host.call('audio.process',{sampleRate:this.sampleRate,samples});}
    }
    class OfflineAudioContext extends BaseAudioContext {
        #channels;
        constructor(channelsOrOptions,length,sampleRate){
            const config=typeof channelsOrOptions==='object'?channelsOrOptions:{numberOfChannels:channelsOrOptions,length,sampleRate};
            super(config);this.#channels=config.numberOfChannels??1;this.length=config.length;this.oncomplete=null;
            // Reuse buffer validation without introducing an offline engine.
            validateBufferSize(this.length,this.#channels);
            this.destination.channelCount=this.#channels;
        }
        // Legacy offline controls return promises that never complete.
        resume(){return new Promise(()=>{});}suspend(){return new Promise(()=>{});}
        startRendering(){return Promise.resolve(new AudioBuffer({numberOfChannels:this.#channels,length:this.length,sampleRate:this.sampleRate}));}
    }
    class SpeechSynthesisUtterance extends EventTarget {
        constructor(text=''){super();this.text=text;}
    }
    // Local utterance payloads only: no speech queue, voices or synthesis engine.
    for(const [name,initial] of Object.entries({text:'',lang:'',pitch:1,rate:1,volume:1,voice:null})){
        const values=new WeakMap();
        Object.defineProperty(SpeechSynthesisUtterance.prototype,name,{
            get(){return values.has(this)?values.get(this):initial;},
            set(value){values.set(this,initial===null?value:typeof initial==='string'?String(value):Number(value));},
            enumerable:true,configurable:true,
        });
    }
    for(const type of ['boundary','end','error','mark','pause','resume','start']){
        const handlers=new WeakMap();
        Object.defineProperty(SpeechSynthesisUtterance.prototype,'on'+type,{
            get(){return handlers.get(this)??null;},
            set(value){handlers.set(this,typeof value==='function'?value:null);},
            enumerable:true,configurable:true,
        });
    }
    Object.defineProperty(global,'speechSynthesis',{value:{},configurable:true});
    for(const [name,value] of Object.entries({BaseAudioContext,AudioContext,OfflineAudioContext,AudioBuffer,AudioParam,AudioNode,AudioScheduledSourceNode,AudioDestinationNode,AudioListener,DynamicsCompressorNode,Worklet,AudioWorklet,GainNode,OscillatorNode,AnalyserNode,SpeechSynthesisUtterance}))Object.defineProperty(global,name,{value,writable:true,configurable:true});
    return ()=>Promise.all([...contexts].map(context=>context.close()));
}
