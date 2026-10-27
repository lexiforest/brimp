/*!
 * Copyright (C) 2017-2021 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */
// Float operations used by WebKit color conversion. The parser and conversion
// graph remain in CSSTools; these dependency adapters preserve Safari precision.
export function multiplyMatrices(matrix, values) {
  const f = Math.fround;
  return [0, 3, 6].map((offset) => {
    let sum = 0;
    for (let i = 0; i < 3; i++) sum = f(sum + f(matrix[offset + i]) * f(values[i]));
    return sum;
  });
}
export function lin_sRGB_channel(value) {
  const f = Math.fround,
    x = Math.abs(f(value));
  return f(
    Math.sign(value) *
      (x <= f(0.04045) ? f(x / f(12.92)) : f(Math.pow(f(f(x + f(0.055)) / f(1.055)), f(2.4)))),
  );
}
export function gam_sRGB_channel(value) {
  const f = Math.fround,
    x = Math.abs(f(value));
  return f(
    Math.sign(value) *
      (x < f(0.0031308) ? f(x * f(12.92)) : f(1.055) * f(Math.pow(x, f(1 / f(2.4)))) - f(0.055)),
  );
}
export function lin_2020_channel(value) {
  const f = Math.fround,
    alpha = f(1.09929682680944),
    beta = f(0.018053968510807),
    gamma = f(0.45);
  value = f(value);
  // Safari applies the nonlinear formula to the signed value. Negative
  // results that become NaN are resolved to zero at the color boundary.
  const result =
    Math.abs(value) < f(beta * 4.5)
      ? f(value / 4.5)
      : f(Math.pow((value + alpha - 1) / alpha, 1 / gamma) * Math.sign(value));
  return Number.isNaN(result) ? 0 : result;
}
export function gam_2020_channel(value) {
  const f = Math.fround,
    alpha = f(1.09929682680944),
    beta = f(0.018053968510807),
    gamma = f(0.45);
  value = f(value);
  const result =
    Math.abs(value) <= beta
      ? f(value * 4.5)
      : f((f(alpha * f(Math.pow(value, gamma))) - (alpha - 1)) * Math.sign(value));
  return Number.isNaN(result) ? 0 : result;
}
export function Lab_to_XYZ(values) {
  const f = Math.fround,
    [l, a, b] = values.map(f);
  const epsilon = f(216 / 24389),
    kappa = f(24389 / 27);
  const y = f(f(l + 16) / 116),
    x = f(y + f(a / 500)),
    z = f(y - f(b / 200));
  const channel = (value) => {
    const cube = f(f(value * value) * value);
    return cube > epsilon ? cube : f(f(116 * value - 16) / kappa);
  };
  const luminance = l > f(kappa * epsilon) ? f((l + 16) / 116) : 0;
  return [
    f(f(0.3457 / 0.3585) * channel(x)),
    l > f(kappa * epsilon) ? f(f(luminance * luminance) * luminance) : f(l / kappa),
    f(f(0.2958 / 0.3585) * channel(z)),
  ];
}
