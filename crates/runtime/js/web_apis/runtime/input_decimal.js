/*
 * Copyright (C) 2012-2015 Google Inc. All rights reserved.
 * Copyright (C) 2022 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above copyright
 * notice, this list of conditions and the following disclaimer in the
 * documentation and/or other materials provided with the distribution.
 *     * Neither the name of Google Inc. nor the names of its contributors may
 * be used to endorse or promote products derived from this software without
 * specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */
// The finite arithmetic needed by Safari range inputs, adapted from WebKit's
// Source/WebCore/platform/Decimal.cpp. This deliberately follows its 18-digit
// arithmetic and serialization, rather than general arbitrary-precision math.
// null represents an invalid/overflow result; there is no public decimal API.
// https://github.com/WebKit/WebKit/blob/main/Source/WebCore/platform/Decimal.cpp
const __inputDecimalMax = 999999999999999999n;
function __inputDecimal(coefficient, exponent = 0) {
    const negative = coefficient < 0n;
    if (negative) coefficient = -coefficient;
    while (coefficient > __inputDecimalMax) { coefficient /= 10n; exponent++; }
    if (exponent < -1023) return {c:0n,e:0};
    if (exponent > 1023) return null;
    return {c:negative ? -coefficient : coefficient,e:exponent};
}
function __parseInputDecimal(text) {
    const match = /^(-?)(?:(\d+)(?:\.(\d*))?|\.(\d+))(?:[eE]([+-]?\d+))?$/.exec(text);
    if (!match || match[0] !== text || !Number.isFinite(Number(text))) return null;
    const integer = (match[2] ?? '').replace(/^0+/, ''), fraction = match[3] ?? match[4] ?? '';
    const extra = Math.max(0, integer.length - 18);
    const fractionDigits = Math.min(fraction.length, Math.max(0, 18 - integer.length));
    const digits = (integer + fraction).slice(0, 18) || '0';
    const coefficient=BigInt(digits) * (match[1] ? -1n : 1n), exponent=Number(match[5] ?? 0) - fractionDigits + extra;
    if (coefficient===0n && (exponent < -1023 || exponent > 1023)) return __inputDecimal(0n);
    return __inputDecimal(coefficient,exponent);
}
function __inputDecimalAdd(a, b, subtract = false) {
    if (!a || !b) return null;
    let exponent = Math.min(a.e,b.e);
    // Align at no more than 18 digits; smaller operands lose low digits exactly
    // as Safari does before adding, including cancellation at huge step bases.
    for (const value of [a,b]) if (value.c !== 0n) {
        const digits = String(value.c < 0n ? -value.c : value.c).length;
        exponent = Math.max(exponent, value.e + digits - 18);
    }
    const scaled = value => value.e >= exponent ? value.c * 10n ** BigInt(value.e-exponent) : value.c / 10n ** BigInt(exponent-value.e);
    return __inputDecimal(scaled(a) + (subtract ? -scaled(b) : scaled(b)), exponent);
}
function __inputDecimalMultiply(a,b) {
    return a && b ? __inputDecimal(a.c*b.c,a.e+b.e) : null;
}
function __inputDecimalDivide(a,b) {
    if (!a || !b || b.c===0n) return null;
    let exponent=a.e-b.e, remainder=a.c<0n?-a.c:a.c, divisor=b.c<0n?-b.c:b.c, result=0n;
    if (remainder===0n) return __inputDecimal(0n,exponent);
    for (;;) {
        while (remainder<divisor && result<__inputDecimalMax/10n) {remainder*=10n;result*=10n;exponent--;}
        if (remainder<divisor) break;
        const quotient=remainder/divisor;
        if (result>__inputDecimalMax-quotient) break;
        result+=quotient;remainder%=divisor;
        if (remainder===0n) break;
    }
    if (remainder>divisor/2n) result++;
    return __inputDecimal((a.c<0n)!==(b.c<0n)?-result:result,exponent);
}
function __inputDecimalRemainder(a, b) {
    const quotient = __inputDecimalDivide(a, b);
    if (!quotient) return null;
    const integral = quotient.e < 0 ? __inputDecimal(quotient.c / 10n ** BigInt(-quotient.e)) : quotient;
    return __inputDecimalAdd(a, __inputDecimalMultiply(integral, b), true);
}
function __inputDecimalRound(value) {
    if (!value || value.e>=0) return value;
    const divisor=10n**BigInt(-value.e), coefficient=value.c<0n?-value.c:value.c;
    const rounded=(coefficient+divisor/2n)/divisor;
    return __inputDecimal(value.c<0n?-rounded:rounded);
}
function __inputDecimalIntegral(value, upward) {
    if (!value || value.c === 0n || value.e >= 0) return value;
    const negative = value.c < 0n, coefficient = negative ? -value.c : value.c;
    const digits = String(coefficient).length, drop = -value.e;
    if (upward && digits <= drop) return __inputDecimal(negative ? 0n : 1n);
    if (!upward && digits < drop) return __inputDecimal(negative ? -1n : 0n);
    const divisor = 10n ** BigInt(drop);
    let integer = coefficient / divisor;
    // WebKit tests the truncated coefficient here, not the original fraction.
    // In particular, floor(-0.66...) is zero and ceil(10.1) can be ten.
    if (negative !== upward && integer % divisor !== 0n) integer++;
    return __inputDecimal(negative ? -integer : integer);
}
function __inputDecimalCompare(a,b) {
    const exponent=Math.min(a.e,b.e);
    const difference=a.c*10n**BigInt(a.e-exponent)-b.c*10n**BigInt(b.e-exponent);
    return difference<0n?-1:difference>0n?1:0;
}
function __serializeInputDecimal(value) {
    let c=value.c<0n?-value.c:value.c,e=value.e;
    if (c===0n) return '0';
    if (e<0) {
        let last=0n;
        while (String(c).length>15) {last=c%10n;c/=10n;e++;}
        if (last>=5n) c++;
        while(e<0 && c%10n===0n) {c/=10n;e++;}
    }
    const digits=String(c), adjusted=e+digits.length-1, sign=value.c<0n?'-':'';
    if (e<=0 && adjusted>=-6) {
        if (e===0) return sign+digits;
        if (adjusted>=0) return sign+digits.slice(0,adjusted+1)+'.'+digits.slice(adjusted+1);
        return sign+'0.'+'0'.repeat(-adjusted-1)+digits;
    }
    const significant=digits.replace(/0+$/,'');
    const mantissa=significant.length>1?significant[0]+'.'+significant.slice(1):significant;
    return sign+mantissa+(adjusted===0?'':(adjusted<0?'e':'e+')+adjusted);
}
