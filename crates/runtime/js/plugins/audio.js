({global,host}, options={}) => {
    const contexts=new Set();
    const validRate=value=>{value=Number(value);if(!Number.isFinite(value)||value<8000||value>96000)throw new DOMException('Unsupported sample rate','NotSupportedError');return value;};
    class AudioBuffer {
        #channels;
        constructor({length,numberOfChannels=1,sampleRate}){if(!Number.isInteger(length)||length<=0||!Number.isInteger(numberOfChannels)||numberOfChannels<1||numberOfChannels>32||length*numberOfChannels>16_000_000)throw new DOMException('Audio buffer size limit','NotSupportedError');Object.defineProperties(this,{length:{value:length},sampleRate:{value:validRate(sampleRate)},numberOfChannels:{value:numberOfChannels}});this.#channels=Array.from({length:numberOfChannels},()=>new Float32Array(length));}
        get duration(){return this.length/this.sampleRate;}
        getChannelData(channel){if(!Number.isInteger(channel)||!this.#channels[channel])throw new DOMException('Invalid channel','IndexSizeError');return this.#channels[channel];}
        copyFromChannel(destination,channel,start=0){if(!(destination instanceof Float32Array))throw new TypeError('Float32Array required');if(!Number.isInteger(start)||start<0)throw new RangeError('Invalid offset');destination.set(this.getChannelData(channel).subarray(start,start+destination.length));}
        copyToChannel(source,channel,start=0){if(!(source instanceof Float32Array))throw new TypeError('Float32Array required');const destination=this.getChannelData(channel);if(!Number.isInteger(start)||start<0)throw new RangeError('Invalid offset');if(start<destination.length)destination.set(source.subarray(0,destination.length-start),start);}
    }
    class AudioParam {
        constructor(value){this.value=value;this.defaultValue=value;this.minValue=-3.4028234663852886e38;this.maxValue=3.4028234663852886e38;this.automationRate='a-rate';}
        setValueAtTime(value,time){if(!Number.isFinite(Number(value))||!Number.isFinite(Number(time))||time<0)throw new RangeError('Invalid audio parameter');this.value=Number(value);return this;}
        cancelScheduledValues(){return this;}cancelAndHoldAtTime(){return this;}
        linearRampToValueAtTime(){throw new DOMException('Audio automation unavailable','NotSupportedError');}
        exponentialRampToValueAtTime(){throw new DOMException('Audio automation unavailable','NotSupportedError');}
    }
    class AudioNode extends EventTarget {
        constructor(context){super();if(!(context instanceof AudioContext))throw new TypeError('AudioContext required');this.context=context;this.channelCount=2;this.channelCountMode='max';this.channelInterpretation='speakers';this.numberOfInputs=1;this.numberOfOutputs=1;this._connections=new Set();}
        connect(destination){if(this.context.state==='closed')throw new DOMException('Context closed','InvalidStateError');if(!(destination instanceof AudioNode)&&!(destination instanceof AudioParam))throw new TypeError('Invalid destination');if(destination.context&&destination.context!==this.context)throw new DOMException('Different context','InvalidAccessError');this._connections.add(destination);return destination instanceof AudioNode?destination:undefined;}
        disconnect(){this._connections.clear();}
    }
    class GainNode extends AudioNode {constructor(context,options={}){super(context);this.gain=new AudioParam(options.gain??1);}}
    class OscillatorNode extends AudioNode {
        constructor(context,options={}){super(context);this.frequency=new AudioParam(options.frequency??440);this.detune=new AudioParam(options.detune??0);this.type=options.type??'sine';this.numberOfInputs=0;}
        start(when=0){if(this._started)throw new DOMException('Already started','InvalidStateError');if(!Number.isFinite(Number(when))||when<0)throw new RangeError('Invalid start');this._started=true;}
        stop(when=0){if(!this._started)throw new DOMException('Not started','InvalidStateError');if(!Number.isFinite(Number(when))||when<0)throw new RangeError('Invalid stop');if(this._timer!==undefined)clearTimeout(this._timer);this._timer=setTimeout(()=>{this.dispatchEvent(new Event('ended'));if(typeof this.onended==='function')this.onended(new Event('ended'));},Math.max(0,(when-this.context.currentTime)*1000));this.context._timers.add(this._timer);}
    }
    class AnalyserNode extends AudioNode {
        #fftSize=2048;
        get fftSize(){return this.#fftSize;}set fftSize(v){v=Number(v);if(!Number.isInteger(v)||v<32||v>32768||(v&(v-1)))throw new DOMException('Invalid FFT size','IndexSizeError');this.#fftSize=v;}
        get frequencyBinCount(){return this.fftSize/2;}
        getFloatFrequencyData(array){array.fill(-Infinity,0,Math.min(array.length,this.frequencyBinCount));}
        getByteFrequencyData(array){array.fill(0,0,Math.min(array.length,this.frequencyBinCount));}
        getFloatTimeDomainData(array){array.fill(0,0,Math.min(array.length,this.fftSize));}
        getByteTimeDomainData(array){array.fill(128,0,Math.min(array.length,this.fftSize));}
    }
    class AudioContext extends EventTarget {
        #state='suspended';#elapsed=0;#started=0;
        constructor(config={}){super();this.sampleRate=validRate(config.sampleRate??options.sampleRate??48000);this.destination=new AudioNode(this);this.destination.numberOfOutputs=0;this._timers=new Set();contexts.add(this);}
        get state(){return this.#state;}get currentTime(){return this.#elapsed+(this.#state==='running'?(performance.now()-this.#started)/1000:0);}
        #change(state){if(this.#state==='closed')throw new DOMException('Context closed','InvalidStateError');this.#elapsed=this.currentTime;this.#started=performance.now();this.#state=state;this.dispatchEvent(new Event('statechange'));if(typeof this.onstatechange==='function')this.onstatechange(new Event('statechange'));}
        async resume(){this.#change('running');}async suspend(){this.#change('suspended');}
        async close(){this.#change('closed');for(const timer of this._timers)clearTimeout(timer);this._timers.clear();contexts.delete(this);}
        createBuffer(channels,length,sampleRate){return new AudioBuffer({numberOfChannels:channels,length,sampleRate});}
        createGain(){return new GainNode(this);}createOscillator(){return new OscillatorNode(this);}createAnalyser(){return new AnalyserNode(this);}
        async decodeAudioData(){throw new DOMException('Audio decoding unavailable','NotSupportedError');}
        processAsync(samples){return host.call('audio.process',{sampleRate:this.sampleRate,samples});}
    }
    for(const [name,value] of Object.entries({AudioContext,AudioBuffer,AudioParam,AudioNode,GainNode,OscillatorNode,AnalyserNode}))Object.defineProperty(global,name,{value,writable:true,configurable:true});
    return ()=>Promise.all([...contexts].map(context=>context.close()));
}
