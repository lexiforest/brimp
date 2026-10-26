var __inputColor = (() => {
  var __defProp = Object.defineProperty;
  var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
  var __getOwnPropNames = Object.getOwnPropertyNames;
  var __hasOwnProp = Object.prototype.hasOwnProperty;
  var __export = (target, all) => {
    for (var name in all)
      __defProp(target, name, { get: all[name], enumerable: true });
  };
  var __copyProps = (to, from, except, desc) => {
    if (from && typeof from === "object" || typeof from === "function") {
      for (let key of __getOwnPropNames(from))
        if (!__hasOwnProp.call(to, key) && key !== except)
          __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
    }
    return to;
  };
  var __toCommonJS = (mod2) => __copyProps(__defProp({}, "__esModule", { value: true }), mod2);

  // input-color.js
  var input_color_exports = {};
  __export(input_color_exports, {
    sanitize: () => sanitize
  });

  // safari-color-math.js
  function multiplyMatrices(matrix, values) {
    const f2 = Math.fround;
    return [0, 3, 6].map((offset) => {
      let sum = 0;
      for (let i3 = 0; i3 < 3; i3++) sum = f2(sum + f2(matrix[offset + i3]) * f2(values[i3]));
      return sum;
    });
  }
  function lin_sRGB_channel(value) {
    const f2 = Math.fround, x2 = Math.abs(f2(value));
    return f2(Math.sign(value) * (x2 <= f2(0.04045) ? f2(x2 / f2(12.92)) : f2(Math.pow(f2(f2(x2 + f2(0.055)) / f2(1.055)), f2(2.4)))));
  }
  function gam_sRGB_channel(value) {
    const f2 = Math.fround, x2 = Math.abs(f2(value));
    return f2(Math.sign(value) * (x2 < f2(31308e-7) ? f2(x2 * f2(12.92)) : f2(1.055) * f2(Math.pow(x2, f2(1 / f2(2.4)))) - f2(0.055)));
  }
  function lin_2020_channel(value) {
    const f2 = Math.fround, alpha2 = f2(1.09929682680944), beta = f2(0.018053968510807), gamma = f2(0.45);
    value = f2(value);
    const result = Math.abs(value) < f2(beta * 4.5) ? f2(value / 4.5) : f2(Math.pow((value + alpha2 - 1) / alpha2, 1 / gamma) * Math.sign(value));
    return Number.isNaN(result) ? 0 : result;
  }
  function gam_2020_channel(value) {
    const f2 = Math.fround, alpha2 = f2(1.09929682680944), beta = f2(0.018053968510807), gamma = f2(0.45);
    value = f2(value);
    const result = Math.abs(value) <= beta ? f2(value * 4.5) : f2((f2(alpha2 * f2(Math.pow(value, gamma))) - (alpha2 - 1)) * Math.sign(value));
    return Number.isNaN(result) ? 0 : result;
  }
  function Lab_to_XYZ(values) {
    const f2 = Math.fround, [l3, a3, b2] = values.map(f2);
    const epsilon = f2(216 / 24389), kappa = f2(24389 / 27);
    const y2 = f2(f2(l3 + 16) / 116), x2 = f2(y2 + f2(a3 / 500)), z2 = f2(y2 - f2(b2 / 200));
    const channel = (value) => {
      const cube = f2(f2(value * value) * value);
      return cube > epsilon ? cube : f2(f2(116 * value - 16) / kappa);
    };
    const luminance2 = l3 > f2(kappa * epsilon) ? f2((l3 + 16) / 116) : 0;
    return [
      f2(f2(0.3457 / 0.3585) * channel(x2)),
      l3 > f2(kappa * epsilon) ? f2(f2(luminance2 * luminance2) * luminance2) : f2(l3 / kappa),
      f2(f2(0.2958 / 0.3585) * channel(z2))
    ];
  }

  // node_modules/@csstools/color-helpers/dist/index.mjs
  function multiplyMatrices2(...args) {
    return multiplyMatrices(...args);
  }
  var _ = [0.955473421488075, -0.02309845494876471, 0.06325924320057072, -0.0283697093338637, 1.0099953980813041, 0.021041441191917323, 0.012314014864481998, -0.020507649298898964, 1.330365926242124];
  function D50_to_D65(t3) {
    return multiplyMatrices2(_, t3);
  }
  var t = [1.0479297925449969, 0.022946870601609652, -0.05019226628920524, 0.02962780877005599, 0.9904344267538799, -0.017073799063418826, -0.009243040646204504, 0.015055191490298152, 0.7518742814281371];
  function D65_to_D50(_3) {
    return multiplyMatrices2(t, _3);
  }
  function HSL_to_sRGB(_3) {
    let t3 = _3[0] % 360;
    const n3 = _3[1] / 100, o3 = _3[2] / 100;
    return t3 < 0 && (t3 += 360), [HSL_to_sRGB_channel(0, t3, n3, o3), HSL_to_sRGB_channel(8, t3, n3, o3), HSL_to_sRGB_channel(4, t3, n3, o3)];
  }
  function HSL_to_sRGB_channel(_3, t3, n3, o3) {
    const e3 = (_3 + t3 / 30) % 12;
    return o3 - n3 * Math.min(o3, 1 - o3) * Math.max(-1, Math.min(e3 - 3, 9 - e3, 1));
  }
  function HWB_to_sRGB(_3) {
    const t3 = _3[0], n3 = _3[1] / 100, o3 = _3[2] / 100;
    if (n3 + o3 >= 1) {
      const _4 = n3 / (n3 + o3);
      return [_4, _4, _4];
    }
    const e3 = HSL_to_sRGB([t3, 100, 50]), r3 = 1 - n3 - o3;
    return [e3[0] * r3 + n3, e3[1] * r3 + n3, e3[2] * r3 + n3];
  }
  function LCH_to_Lab(_3) {
    const t3 = _3[2] * Math.PI / 180;
    return [_3[0], _3[1] * Math.cos(t3), _3[1] * Math.sin(t3)];
  }
  function Lab_to_LCH(_3) {
    const t3 = Math.sqrt(Math.pow(_3[1], 2) + Math.pow(_3[2], 2));
    let n3 = 180 * Math.atan2(_3[2], _3[1]) / Math.PI;
    return n3 < 0 && (n3 += 360), t3 <= 15e-4 && (n3 = NaN), [_3[0], t3, n3];
  }
  var n = [0.3457 / 0.3585, 1, 0.2958 / 0.3585];
  function Lab_to_XYZ2(...args) {
    return Lab_to_XYZ(...args);
  }
  function OKLCH_to_OKLab(_3) {
    const t3 = _3[2] * Math.PI / 180;
    return [_3[0], _3[1] * Math.cos(t3), _3[1] * Math.sin(t3)];
  }
  function OKLab_to_OKLCH(_3) {
    const t3 = Math.sqrt(_3[1] ** 2 + _3[2] ** 2);
    let n3 = 180 * Math.atan2(_3[2], _3[1]) / Math.PI;
    return n3 < 0 && (n3 += 360), t3 <= 4e-6 && (n3 = NaN), [_3[0], t3, n3];
  }
  var o = [1.2268798758459243, -0.5578149944602171, 0.2813910456659647, -0.0405757452148008, 1.112286803280317, -0.0717110580655164, -0.0763729366746601, -0.4214933324022432, 1.5869240198367816];
  var e = [1, 0.3963377773761749, 0.2158037573099136, 1, -0.1055613458156586, -0.0638541728258133, 1, -0.0894841775298119, -1.2914855480194092];
  function OKLab_to_XYZ(_3) {
    const t3 = multiplyMatrices2(e, _3);
    return multiplyMatrices2(o, [t3[0] ** 3, t3[1] ** 3, t3[2] ** 3]);
  }
  function XYZ_to_Lab(_3) {
    const t3 = compute_f(_3[0] / n[0]), o3 = compute_f(_3[1] / n[1]);
    return [116 * o3 - 16, 500 * (t3 - o3), 200 * (o3 - compute_f(_3[2] / n[2]))];
  }
  var r = 216 / 24389;
  var a = 24389 / 27;
  function compute_f(_3) {
    return _3 > r ? Math.cbrt(_3) : (a * _3 + 16) / 116;
  }
  var i = [0.819022437996703, 0.3619062600528904, -0.1288737815209879, 0.0329836539323885, 0.9292868615863434, 0.0361446663506424, 0.0481771893596242, 0.2642395317527308, 0.6335478284694309];
  var l = [0.210454268309314, 0.7936177747023054, -0.0040720430116193, 1.9779985324311684, -2.42859224204858, 0.450593709617411, 0.0259040424655478, 0.7827717124575296, -0.8086757549230774];
  function XYZ_to_OKLab(_3) {
    const t3 = multiplyMatrices2(i, _3);
    return multiplyMatrices2(l, [Math.cbrt(t3[0]), Math.cbrt(t3[1]), Math.cbrt(t3[2])]);
  }
  var u = [30757411 / 17917100, -6372589 / 17917100, -4539589 / 17917100, -0.666684351832489, 1.616481236634939, 467509 / 29648200, 792561 / 44930125, -1921689 / 44930125, 0.942103121235474];
  function XYZ_to_lin_2020(_3) {
    return multiplyMatrices2(u, _3);
  }
  var c = [446124 / 178915, -333277 / 357830, -72051 / 178915, -14852 / 17905, 63121 / 35810, 423 / 17905, 11844 / 330415, -50337 / 660830, 316169 / 330415];
  function XYZ_to_lin_P3(_3) {
    return multiplyMatrices2(c, _3);
  }
  var s = [1.3457868816471583, -0.25557208737979464, -0.05110186497554526, -0.5446307051249019, 1.5082477428451468, 0.02052744743642139, 0, 0, 1.2119675456389452];
  function XYZ_to_lin_ProPhoto(_3) {
    return multiplyMatrices2(s, _3);
  }
  var X = [1829569 / 896150, -506331 / 896150, -308931 / 896150, -851781 / 878810, 1648619 / 878810, 36519 / 878810, 16779 / 1248040, -147721 / 1248040, 1266979 / 1248040];
  function XYZ_to_lin_a98rgb(_3) {
    return multiplyMatrices2(X, _3);
  }
  var Y = [12831 / 3959, -329 / 214, -1974 / 3959, -851781 / 878810, 1648619 / 878810, 36519 / 878810, 705 / 12673, -2585 / 12673, 705 / 667];
  function XYZ_to_lin_sRGB(_3) {
    return multiplyMatrices2(Y, _3);
  }
  function gam_2020(_3) {
    return [gam_2020_channel2(_3[0]), gam_2020_channel2(_3[1]), gam_2020_channel2(_3[2])];
  }
  function gam_2020_channel2(...args) {
    return gam_2020_channel(...args);
  }
  function gam_sRGB(_3) {
    return [gam_sRGB_channel2(_3[0]), gam_sRGB_channel2(_3[1]), gam_sRGB_channel2(_3[2])];
  }
  function gam_sRGB_channel2(...args) {
    return gam_sRGB_channel(...args);
  }
  function gam_P3(_3) {
    return gam_sRGB(_3);
  }
  function gam_ProPhoto(_3) {
    return [gam_ProPhoto_channel(_3[0]), gam_ProPhoto_channel(_3[1]), gam_ProPhoto_channel(_3[2])];
  }
  var Z = 1 / 512;
  function gam_ProPhoto_channel(_3) {
    const t3 = _3 < 0 ? -1 : 1, n3 = Math.abs(_3);
    return n3 >= Z ? t3 * Math.pow(n3, 1 / 1.8) : 16 * _3;
  }
  function gam_a98rgb(_3) {
    return [gam_a98rgb_channel(_3[0]), gam_a98rgb_channel(_3[1]), gam_a98rgb_channel(_3[2])];
  }
  function gam_a98rgb_channel(_3) {
    const t3 = _3 < 0 ? -1 : 1, n3 = Math.abs(_3);
    return t3 * Math.pow(n3, 256 / 563);
  }
  function lin_2020(_3) {
    return [lin_2020_channel2(_3[0]), lin_2020_channel2(_3[1]), lin_2020_channel2(_3[2])];
  }
  function lin_2020_channel2(...args) {
    return lin_2020_channel(...args);
  }
  var D = [63426534 / 99577255, 20160776 / 139408157, 47086771 / 278816314, 26158966 / 99577255, 0.677998071518871, 8267143 / 139408157, 0, 19567812 / 697040785, 1.0609850577107909];
  function lin_2020_to_XYZ(_3) {
    return multiplyMatrices2(D, _3);
  }
  function lin_sRGB(_3) {
    return [lin_sRGB_channel2(_3[0]), lin_sRGB_channel2(_3[1]), lin_sRGB_channel2(_3[2])];
  }
  function lin_sRGB_channel2(...args) {
    return lin_sRGB_channel(...args);
  }
  function lin_P3(_3) {
    return lin_sRGB(_3);
  }
  var h = [608311 / 1250200, 189793 / 714400, 198249 / 1000160, 35783 / 156275, 247089 / 357200, 198249 / 2500400, 0, 32229 / 714400, 5220557 / 5000800];
  function lin_P3_to_XYZ(_3) {
    return multiplyMatrices2(h, _3);
  }
  function lin_ProPhoto(_3) {
    return [lin_ProPhoto_channel(_3[0]), lin_ProPhoto_channel(_3[1]), lin_ProPhoto_channel(_3[2])];
  }
  var f = 16 / 512;
  function lin_ProPhoto_channel(_3) {
    const t3 = _3 < 0 ? -1 : 1, n3 = Math.abs(_3);
    return n3 <= f ? _3 / 16 : t3 * Math.pow(n3, 1.8);
  }
  var m = [0.7977666449006423, 0.13518129740053308, 0.0313477341283922, 0.2880748288194013, 0.711835234241873, 8993693872564e-17, 0, 0, 0.8251046025104602];
  function lin_ProPhoto_to_XYZ(_3) {
    return multiplyMatrices2(m, _3);
  }
  function lin_a98rgb(_3) {
    return [lin_a98rgb_channel(_3[0]), lin_a98rgb_channel(_3[1]), lin_a98rgb_channel(_3[2])];
  }
  function lin_a98rgb_channel(_3) {
    const t3 = _3 < 0 ? -1 : 1, n3 = Math.abs(_3);
    return t3 * Math.pow(n3, 563 / 256);
  }
  var b = [573536 / 994567, 263643 / 1420810, 187206 / 994567, 591459 / 1989134, 6239551 / 9945670, 374412 / 4972835, 53769 / 1989134, 351524 / 4972835, 4929758 / 4972835];
  function lin_a98rgb_to_XYZ(_3) {
    return multiplyMatrices2(b, _3);
  }
  var g = [506752 / 1228815, 87881 / 245763, 12673 / 70218, 87098 / 409605, 175762 / 245763, 12673 / 175545, 7918 / 409605, 87881 / 737289, 1001167 / 1053270];
  function lin_sRGB_to_XYZ(_3) {
    return multiplyMatrices2(g, _3);
  }
  function sRGB_to_HSL(_3) {
    const t3 = _3[0], n3 = _3[1], o3 = _3[2], e3 = Math.max(t3, n3, o3), r3 = Math.min(t3, n3, o3), a3 = (r3 + e3) / 2, i3 = e3 - r3;
    let l3 = Number.NaN, u3 = 0;
    if (0 !== Math.round(1e5 * i3)) {
      const _4 = Math.round(1e5 * a3);
      switch (u3 = 0 === _4 || 1e5 === _4 ? 0 : (e3 - a3) / Math.min(a3, 1 - a3), e3) {
        case t3:
          l3 = (n3 - o3) / i3 + (n3 < o3 ? 6 : 0);
          break;
        case n3:
          l3 = (o3 - t3) / i3 + 2;
          break;
        case o3:
          l3 = (t3 - n3) / i3 + 4;
      }
      l3 *= 60;
    }
    u3 < 0 && (l3 += 180, u3 = Math.abs(u3)), l3 >= 360 && (l3 -= 360);
    return u3 <= 1e-5 && (l3 = NaN), [l3, 100 * u3, 100 * a3];
  }
  function sRGB_to_Hue(_3) {
    const t3 = _3[0], n3 = _3[1], o3 = _3[2], e3 = Math.max(t3, n3, o3), r3 = Math.min(t3, n3, o3);
    let a3 = Number.NaN;
    const i3 = e3 - r3;
    if (0 !== i3) {
      switch (e3) {
        case t3:
          a3 = (n3 - o3) / i3 + (n3 < o3 ? 6 : 0);
          break;
        case n3:
          a3 = (o3 - t3) / i3 + 2;
          break;
        case o3:
          a3 = (t3 - n3) / i3 + 4;
      }
      a3 *= 60;
    }
    return a3 >= 360 && (a3 -= 360), a3;
  }
  function sRGB_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = lin_sRGB(t3), t3 = lin_sRGB_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function HSL_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = HSL_to_sRGB(t3), t3 = lin_sRGB(t3), t3 = lin_sRGB_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function HWB_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = HWB_to_sRGB(t3), t3 = lin_sRGB(t3), t3 = lin_sRGB_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function Lab_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = Lab_to_XYZ2(t3), t3;
  }
  function XYZ_D50_to_Lab(_3) {
    let t3 = _3;
    return t3 = XYZ_to_Lab(t3), t3;
  }
  function LCH_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = LCH_to_Lab(t3), t3 = Lab_to_XYZ2(t3), t3;
  }
  function XYZ_D50_to_LCH(_3) {
    let t3 = _3;
    return t3 = XYZ_to_Lab(t3), t3 = Lab_to_LCH(t3), t3;
  }
  function OKLab_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = OKLab_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function OKLCH_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = OKLCH_to_OKLab(t3), t3 = OKLab_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function lin_sRGB_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = lin_sRGB_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function a98_RGB_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = lin_a98rgb(t3), t3 = lin_a98rgb_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function P3_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = lin_P3(t3), t3 = lin_P3_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function lin_P3_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = lin_P3_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function rec_2020_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = lin_2020(t3), t3 = lin_2020_to_XYZ(t3), t3 = D65_to_D50(t3), t3;
  }
  function ProPhoto_RGB_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = lin_ProPhoto(t3), t3 = lin_ProPhoto_to_XYZ(t3), t3;
  }
  function XYZ_D50_to_ProPhoto(_3) {
    let t3 = _3;
    return t3 = XYZ_to_lin_ProPhoto(t3), t3 = gam_ProPhoto(t3), t3;
  }
  function XYZ_D65_to_XYZ_D50(_3) {
    let t3 = _3;
    return t3 = D65_to_D50(t3), t3;
  }
  function XYZ_D50_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = D50_to_D65(t3), t3;
  }
  function XYZ_D50_to_XYZ_D50(_3) {
    return _3;
  }
  function XYZ_D65_to_XYZ_D65(_3) {
    return _3;
  }
  function sRGB_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = lin_sRGB(t3), t3 = lin_sRGB_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_sRGB(_3) {
    let t3 = _3;
    return t3 = XYZ_to_lin_sRGB(t3), t3 = gam_sRGB(t3), t3;
  }
  function HSL_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = HSL_to_sRGB(t3), t3 = lin_sRGB(t3), t3 = lin_sRGB_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_HSL(_3) {
    let t3 = _3;
    return t3 = XYZ_to_lin_sRGB(t3), t3 = gam_sRGB(t3), t3 = sRGB_to_HSL(t3), t3;
  }
  function HWB_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = HWB_to_sRGB(t3), t3 = lin_sRGB(t3), t3 = lin_sRGB_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_HWB(_3) {
    let t3 = _3;
    t3 = XYZ_to_lin_sRGB(t3);
    const n3 = gam_sRGB(t3), o3 = Math.min(n3[0], n3[1], n3[2]), e3 = 1 - Math.max(n3[0], n3[1], n3[2]);
    let r3 = sRGB_to_Hue(n3);
    return o3 + e3 >= 0.99999 && (r3 = NaN), [r3, 100 * o3, 100 * e3];
  }
  function Lab_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = Lab_to_XYZ2(t3), t3 = D50_to_D65(t3), t3;
  }
  function LCH_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = LCH_to_Lab(t3), t3 = Lab_to_XYZ2(t3), t3 = D50_to_D65(t3), t3;
  }
  function OKLab_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = OKLab_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_OKLab(_3) {
    let t3 = _3;
    return t3 = XYZ_to_OKLab(t3), t3;
  }
  function OKLCH_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = OKLCH_to_OKLab(t3), t3 = OKLab_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_OKLCH(_3) {
    let t3 = _3;
    return t3 = XYZ_to_OKLab(t3), t3 = OKLab_to_OKLCH(t3), t3;
  }
  function lin_sRGB_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = lin_sRGB_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_lin_sRGB(_3) {
    let t3 = _3;
    return t3 = XYZ_to_lin_sRGB(t3), t3;
  }
  function a98_RGB_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = lin_a98rgb(t3), t3 = lin_a98rgb_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_a98_RGB(_3) {
    let t3 = _3;
    return t3 = XYZ_to_lin_a98rgb(t3), t3 = gam_a98rgb(t3), t3;
  }
  function P3_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = lin_P3(t3), t3 = lin_P3_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_P3(_3) {
    let t3 = _3;
    return t3 = XYZ_to_lin_P3(t3), t3 = gam_P3(t3), t3;
  }
  function lin_P3_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = lin_P3_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_lin_P3(_3) {
    let t3 = _3;
    return t3 = XYZ_to_lin_P3(t3), t3;
  }
  function rec_2020_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = lin_2020(t3), t3 = lin_2020_to_XYZ(t3), t3;
  }
  function XYZ_D65_to_rec_2020(_3) {
    let t3 = _3;
    return t3 = XYZ_to_lin_2020(t3), t3 = gam_2020(t3), t3;
  }
  function ProPhoto_RGB_to_XYZ_D65(_3) {
    let t3 = _3;
    return t3 = lin_ProPhoto(t3), t3 = lin_ProPhoto_to_XYZ(t3), t3 = D50_to_D65(t3), t3;
  }
  function inGamut(_3) {
    return _3[0] >= -1e-4 && _3[0] <= 1.0001 && _3[1] >= -1e-4 && _3[1] <= 1.0001 && _3[2] >= -1e-4 && _3[2] <= 1.0001;
  }
  function clip(_3) {
    return [_3[0] < 0 ? 0 : _3[0] > 1 ? 1 : _3[0], _3[1] < 0 ? 0 : _3[1] > 1 ? 1 : _3[1], _3[2] < 0 ? 0 : _3[2] > 1 ? 1 : _3[2]];
  }
  function mapGamutRayTrace(_3, t3, n3) {
    const o3 = _3[0], e3 = _3[2];
    let r3 = t3(_3);
    const a3 = t3([o3, 0, e3]);
    for (let _4 = 0; _4 < 4; _4++) {
      if (_4 > 0) {
        const _5 = n3(r3);
        _5[0] = o3, _5[2] = e3, r3 = t3(_5);
      }
      const i3 = rayTraceBox(a3, r3);
      if (!i3) break;
      r3 = i3;
    }
    return clip(r3);
  }
  function rayTraceBox(_3, t3) {
    let n3 = 1 / 0, o3 = -1 / 0;
    const e3 = [0, 0, 0];
    for (let r3 = 0; r3 < 3; r3++) {
      const a3 = _3[r3], i3 = t3[r3] - a3;
      e3[r3] = i3;
      const l3 = 0, u3 = 1;
      if (Math.abs(i3) > 1e-12) {
        const _4 = 1 / i3, t4 = (l3 - a3) * _4, e4 = (u3 - a3) * _4;
        o3 = Math.max(Math.min(t4, e4), o3), n3 = Math.min(Math.max(t4, e4), n3);
      } else if (a3 < l3 || a3 > u3) return false;
    }
    return !(o3 > n3 || n3 < 0) && (o3 < 0 && (o3 = n3), !!isFinite(o3) && [_3[0] + e3[0] * o3, _3[1] + e3[1] * o3, _3[2] + e3[2] * o3]);
  }
  var G = { aliceblue: [240, 248, 255], antiquewhite: [250, 235, 215], aqua: [0, 255, 255], aquamarine: [127, 255, 212], azure: [240, 255, 255], beige: [245, 245, 220], bisque: [255, 228, 196], black: [0, 0, 0], blanchedalmond: [255, 235, 205], blue: [0, 0, 255], blueviolet: [138, 43, 226], brown: [165, 42, 42], burlywood: [222, 184, 135], cadetblue: [95, 158, 160], chartreuse: [127, 255, 0], chocolate: [210, 105, 30], coral: [255, 127, 80], cornflowerblue: [100, 149, 237], cornsilk: [255, 248, 220], crimson: [220, 20, 60], cyan: [0, 255, 255], darkblue: [0, 0, 139], darkcyan: [0, 139, 139], darkgoldenrod: [184, 134, 11], darkgray: [169, 169, 169], darkgreen: [0, 100, 0], darkgrey: [169, 169, 169], darkkhaki: [189, 183, 107], darkmagenta: [139, 0, 139], darkolivegreen: [85, 107, 47], darkorange: [255, 140, 0], darkorchid: [153, 50, 204], darkred: [139, 0, 0], darksalmon: [233, 150, 122], darkseagreen: [143, 188, 143], darkslateblue: [72, 61, 139], darkslategray: [47, 79, 79], darkslategrey: [47, 79, 79], darkturquoise: [0, 206, 209], darkviolet: [148, 0, 211], deeppink: [255, 20, 147], deepskyblue: [0, 191, 255], dimgray: [105, 105, 105], dimgrey: [105, 105, 105], dodgerblue: [30, 144, 255], firebrick: [178, 34, 34], floralwhite: [255, 250, 240], forestgreen: [34, 139, 34], fuchsia: [255, 0, 255], gainsboro: [220, 220, 220], ghostwhite: [248, 248, 255], gold: [255, 215, 0], goldenrod: [218, 165, 32], gray: [128, 128, 128], green: [0, 128, 0], greenyellow: [173, 255, 47], grey: [128, 128, 128], honeydew: [240, 255, 240], hotpink: [255, 105, 180], indianred: [205, 92, 92], indigo: [75, 0, 130], ivory: [255, 255, 240], khaki: [240, 230, 140], lavender: [230, 230, 250], lavenderblush: [255, 240, 245], lawngreen: [124, 252, 0], lemonchiffon: [255, 250, 205], lightblue: [173, 216, 230], lightcoral: [240, 128, 128], lightcyan: [224, 255, 255], lightgoldenrodyellow: [250, 250, 210], lightgray: [211, 211, 211], lightgreen: [144, 238, 144], lightgrey: [211, 211, 211], lightpink: [255, 182, 193], lightsalmon: [255, 160, 122], lightseagreen: [32, 178, 170], lightskyblue: [135, 206, 250], lightslategray: [119, 136, 153], lightslategrey: [119, 136, 153], lightsteelblue: [176, 196, 222], lightyellow: [255, 255, 224], lime: [0, 255, 0], limegreen: [50, 205, 50], linen: [250, 240, 230], magenta: [255, 0, 255], maroon: [128, 0, 0], mediumaquamarine: [102, 205, 170], mediumblue: [0, 0, 205], mediumorchid: [186, 85, 211], mediumpurple: [147, 112, 219], mediumseagreen: [60, 179, 113], mediumslateblue: [123, 104, 238], mediumspringgreen: [0, 250, 154], mediumturquoise: [72, 209, 204], mediumvioletred: [199, 21, 133], midnightblue: [25, 25, 112], mintcream: [245, 255, 250], mistyrose: [255, 228, 225], moccasin: [255, 228, 181], navajowhite: [255, 222, 173], navy: [0, 0, 128], oldlace: [253, 245, 230], olive: [128, 128, 0], olivedrab: [107, 142, 35], orange: [255, 165, 0], orangered: [255, 69, 0], orchid: [218, 112, 214], palegoldenrod: [238, 232, 170], palegreen: [152, 251, 152], paleturquoise: [175, 238, 238], palevioletred: [219, 112, 147], papayawhip: [255, 239, 213], peachpuff: [255, 218, 185], peru: [205, 133, 63], pink: [255, 192, 203], plum: [221, 160, 221], powderblue: [176, 224, 230], purple: [128, 0, 128], rebeccapurple: [102, 51, 153], red: [255, 0, 0], rosybrown: [188, 143, 143], royalblue: [65, 105, 225], saddlebrown: [139, 69, 19], salmon: [250, 128, 114], sandybrown: [244, 164, 96], seagreen: [46, 139, 87], seashell: [255, 245, 238], sienna: [160, 82, 45], silver: [192, 192, 192], skyblue: [135, 206, 235], slateblue: [106, 90, 205], slategray: [112, 128, 144], slategrey: [112, 128, 144], snow: [255, 250, 250], springgreen: [0, 255, 127], steelblue: [70, 130, 180], tan: [210, 180, 140], teal: [0, 128, 128], thistle: [216, 191, 216], tomato: [255, 99, 71], turquoise: [64, 224, 208], violet: [238, 130, 238], wheat: [245, 222, 179], white: [255, 255, 255], whitesmoke: [245, 245, 245], yellow: [255, 255, 0], yellowgreen: [154, 205, 50] };
  function luminance(_3) {
    const [t3, n3, o3] = _3.map((_4) => _4 <= 0.04045 ? _4 / 12.92 : Math.pow((_4 + 0.055) / 1.055, 2.4));
    return 0.2126 * t3 + 0.7152 * n3 + 0.0722 * o3;
  }
  function contrast_ratio_wcag_2_1(_3, t3) {
    const n3 = luminance(_3), o3 = luminance(t3);
    return (Math.max(n3, o3) + 0.05) / (Math.min(n3, o3) + 0.05);
  }

  // safari-color-conversion.js
  function directSRGBConversion(color2, destination) {
    const family = ["rgb", "hex", "srgb", "hsl", "hwb"];
    if (!family.includes(color2.colorNotation) || !family.includes(destination)) return null;
    const channels = color2.channels.map((value) => Number.isNaN(value) ? 0 : value);
    const rgb2 = color2.colorNotation === "hsl" ? HSL_to_sRGB(channels) : color2.colorNotation === "hwb" ? HWB_to_sRGB(channels) : channels;
    if (destination === "hsl") return sRGB_to_HSL(rgb2);
    if (destination === "hwb") {
      const white = Math.min(...rgb2), black = 1 - Math.max(...rgb2);
      return [white + black >= 0.99999 ? NaN : sRGB_to_Hue(rgb2), white * 100, black * 100];
    }
    return rgb2;
  }

  // node_modules/@csstools/css-tokenizer/dist/index.mjs
  var ParseError = class extends Error {
    sourceStart;
    sourceEnd;
    parserState;
    constructor(e3, n3, t3, o3) {
      super(e3), this.name = "ParseError", this.sourceStart = n3, this.sourceEnd = t3, this.parserState = o3;
    }
  };
  var ParseErrorWithToken = class extends ParseError {
    token;
    constructor(e3, n3, t3, o3, r3) {
      super(e3, n3, t3, o3), this.token = r3;
    }
  };
  var e2 = { UnexpectedNewLineInString: "Unexpected newline while consuming a string token.", UnexpectedEOFInString: "Unexpected EOF while consuming a string token.", UnexpectedEOFInComment: "Unexpected EOF while consuming a comment.", UnexpectedEOFInURL: "Unexpected EOF while consuming a url token.", UnexpectedEOFInEscapedCodePoint: "Unexpected EOF while consuming an escaped code point.", UnexpectedCharacterInURL: "Unexpected character while consuming a url token.", InvalidEscapeSequenceInURL: "Invalid escape sequence while consuming a url token.", InvalidEscapeSequenceAfterBackslash: 'Invalid escape sequence after "\\"' };
  var n2 = "undefined" != typeof globalThis && "structuredClone" in globalThis;
  function stringify(...e3) {
    if (!e3 || !e3.length) return "";
    let n3;
    n3 = Array.isArray(e3) && Array.isArray(e3[0]) && Array.isArray(e3[0][0]) ? e3[0] : e3;
    let t3 = "";
    for (let e4 = 0; e4 < n3.length; e4++) t3 += n3[e4][1];
    return t3;
  }
  var t2 = 13;
  var o2 = 45;
  var r2 = 10;
  var i2 = 43;
  var c2 = 65533;
  var s2 = 92;
  var u2;
  var a2;
  var d;
  function mirrorVariantType(e3) {
    switch (e3) {
      case u2.OpenParen:
        return u2.CloseParen;
      case u2.CloseParen:
        return u2.OpenParen;
      case u2.OpenCurly:
        return u2.CloseCurly;
      case u2.CloseCurly:
        return u2.OpenCurly;
      case u2.OpenSquare:
        return u2.CloseSquare;
      case u2.CloseSquare:
        return u2.OpenSquare;
      default:
        return null;
    }
  }
  function mirrorVariant(e3) {
    switch (e3[0]) {
      case u2.OpenParen:
        return [u2.CloseParen, ")", -1, -1, void 0];
      case u2.CloseParen:
        return [u2.OpenParen, "(", -1, -1, void 0];
      case u2.OpenCurly:
        return [u2.CloseCurly, "}", -1, -1, void 0];
      case u2.CloseCurly:
        return [u2.OpenCurly, "{", -1, -1, void 0];
      case u2.OpenSquare:
        return [u2.CloseSquare, "]", -1, -1, void 0];
      case u2.CloseSquare:
        return [u2.OpenSquare, "[", -1, -1, void 0];
      default:
        return null;
    }
  }
  function isHexDigitCodePoint(e3) {
    return e3 >= 48 && e3 <= 57 || e3 >= 97 && e3 <= 102 || e3 >= 65 && e3 <= 70;
  }
  function isIdentStartCodePoint(e3) {
    return e3 >= 97 && e3 <= 122 || e3 >= 65 && e3 <= 90 || 95 === e3 || isNonASCII_IdentCodePoint(e3);
  }
  function isIdentCodePoint(e3) {
    return e3 >= 97 && e3 <= 122 || e3 >= 65 && e3 <= 90 || e3 >= 48 && e3 <= 57 || e3 === o2 || 95 === e3 || isNonASCII_IdentCodePoint(e3);
  }
  function isNonASCII_IdentCodePoint(e3) {
    return e3 < 183 ? 0 === e3 : 183 === e3 || 8204 === e3 || 8205 === e3 || 8255 === e3 || 8256 === e3 || 8204 === e3 || (192 <= e3 && e3 <= 214 || 216 <= e3 && e3 <= 246 || 248 <= e3 && e3 <= 893 || 895 <= e3 && e3 <= 8191 || 8304 <= e3 && e3 <= 8591 || 11264 <= e3 && e3 <= 12271 || 12289 <= e3 && e3 <= 55295 || 63744 <= e3 && e3 <= 64975 || 65008 <= e3 && e3 <= 65533 || (0 === e3 || (e3 >= 55296 && e3 <= 57343 || e3 >= 65536)));
  }
  function isNonPrintableCodePoint(e3) {
    return 11 === e3 || 127 === e3 || 0 <= e3 && e3 <= 8 || 14 <= e3 && e3 <= 31;
  }
  function isNewLine(e3) {
    return e3 === r2 || e3 === t2 || 12 === e3;
  }
  function isWhitespace(e3) {
    return 32 === e3 || e3 === r2 || 9 === e3 || e3 === t2 || 12 === e3;
  }
  function isSurrogate(e3) {
    return e3 >= 55296 && e3 <= 57343;
  }
  function tokenize(e3, n3) {
    const t3 = tokenizer(e3, n3), o3 = [];
    for (; !t3.endOfFile(); ) o3.push(t3.nextToken());
    return o3.push(t3.nextToken()), o3;
  }
  function tokenizer(n3, l3) {
    const C = n3.css.valueOf(), f2 = n3.unicodeRangesAllowed ?? false;
    let m2 = 0, h2 = 0, k3 = -1, P2 = false;
    const p = l3?.onParseError ?? noop;
    function consumeIdentSequence() {
      const e3 = m2;
      for (; ; ) {
        const e4 = C.charCodeAt(m2);
        if (e4 >= 65 && e4 <= 90 || e4 >= 97 && e4 <= 122 || e4 >= 48 && e4 <= 57 || e4 === o2 || 95 === e4) {
          m2 += 1;
          continue;
        }
        const n5 = C.codePointAt(m2) ?? -1;
        if (0 === n5 || n5 >= 55296 && n5 <= 57343 || n5 === s2 && !isNewLine(C.codePointAt(m2 + 1) ?? -1)) break;
        if (!isNonASCII_IdentCodePoint(n5)) break;
        m2 += +(n5 > 65535) + 1;
      }
      const n4 = C.codePointAt(m2) ?? -1;
      if (0 !== n4 && !(n4 >= 55296 && n4 <= 57343) && (n4 !== s2 || isNewLine(C.codePointAt(m2 + 1) ?? -1))) return P2 = true, k3 = m2 - 1, C.slice(e3, m2);
      P2 = false, m2 = e3, k3 = -1;
      let t3 = "";
      for (; ; ) {
        const e4 = C.codePointAt(m2) ?? -1;
        if (0 === e4 || isSurrogate(e4)) t3 += String.fromCharCode(c2), m2 += +(e4 > 65535) + 1, k3 = m2 - 1;
        else if (e4 >= 65 && e4 <= 90 || e4 >= 97 && e4 <= 122 || e4 >= 48 && e4 <= 57 || e4 === o2 || 95 === e4 || isNonASCII_IdentCodePoint(e4)) t3 += e4 > 65535 ? String.fromCodePoint(e4) : String.fromCharCode(e4), m2 += +(e4 > 65535) + 1, k3 = m2 - 1;
        else {
          if (C.codePointAt(m2) !== s2 || isNewLine(C.codePointAt(m2 + 1) ?? -1)) return t3;
          m2 += 1, k3 = m2 - 1, t3 += String.fromCodePoint(consumeEscapedCodePoint());
        }
      }
    }
    function consumeEscapedCodePoint() {
      const n4 = C.codePointAt(m2);
      if (void 0 === n4) return p(new ParseError(e2.UnexpectedEOFInEscapedCodePoint, h2, k3, ["4.3.7. Consume an escaped code point", "Unexpected EOF"])), c2;
      if (m2 += +(n4 > 65535) + 1, k3 = m2 - 1, isHexDigitCodePoint(n4)) {
        const e3 = [n4];
        let o3;
        for (; void 0 !== (o3 = C.codePointAt(m2)) && isHexDigitCodePoint(o3) && e3.length < 6; ) e3.push(o3), m2 += 1, k3 = m2 - 1;
        isWhitespace(C.codePointAt(m2) ?? -1) && (C.codePointAt(m2) === t2 && C.codePointAt(m2 + 1) === r2 && (m2 += 1), m2 += 1, k3 = m2 - 1);
        const i3 = parseInt(String.fromCodePoint(...e3), 16);
        return 0 === i3 || isSurrogate(i3) || i3 > 1114111 ? c2 : i3;
      }
      return 0 === n4 || isSurrogate(n4) ? c2 : n4;
    }
    function consumeBadURL() {
      for (; ; ) {
        const e3 = C.codePointAt(m2);
        if (void 0 === e3) return;
        if (41 === e3) return m2 += 1, void (k3 = m2 - 1);
        e3 !== s2 || isNewLine(C.codePointAt(m2 + 1) ?? -1) ? (m2 += 1, k3 = m2 - 1) : (m2 += 1, k3 = m2 - 1, consumeEscapedCodePoint());
      }
    }
    function consumeComment2() {
      for (m2 += 2, k3 = m2 - 1; ; ) {
        const n4 = C.codePointAt(m2);
        if (void 0 === n4) {
          const n5 = [u2.Comment, C.slice(h2, k3 + 1), h2, k3, void 0];
          return p(new ParseErrorWithToken(e2.UnexpectedEOFInComment, h2, k3, ["4.3.2. Consume comments", "Unexpected EOF"], n5)), n5;
        }
        if (m2 += 1, k3 = m2 - 1, 42 === n4 && void 0 !== C.codePointAt(m2) && 47 === C.codePointAt(m2)) {
          m2 += 1, k3 = m2 - 1;
          break;
        }
      }
      return [u2.Comment, C.slice(h2, k3 + 1), h2, k3, void 0];
    }
    function consumeStringToken() {
      let n4 = "";
      const o3 = C.codePointAt(m2);
      for (m2 += 1, k3 = m2 - 1; ; ) {
        const i3 = C.charCodeAt(m2);
        if (m2 >= C.length) {
          const t3 = [u2.String, C.slice(h2, k3 + 1), h2, k3, { value: n4 }];
          return p(new ParseErrorWithToken(e2.UnexpectedEOFInString, h2, k3, ["4.3.5. Consume a string token", "Unexpected EOF"], t3)), t3;
        }
        if (m2 += 1, k3 = m2 - 1, isNewLine(i3)) {
          m2 -= 1, k3 = m2 - 1;
          const n5 = [u2.BadString, C.slice(h2, k3 + 1), h2, k3, void 0];
          return p(new ParseErrorWithToken(e2.UnexpectedNewLineInString, h2, C.codePointAt(m2) === t2 && C.codePointAt(m2 + 1) === r2 ? k3 + 2 : k3 + 1, ["4.3.5. Consume a string token", "Unexpected newline"], n5)), n5;
        }
        if (i3 === o3) return [u2.String, C.slice(h2, k3 + 1), h2, k3, { value: n4 }];
        if (i3 !== s2) if (0 !== i3) if (i3 < 128) n4 += String.fromCharCode(i3);
        else {
          if (i3 >= 55296 && i3 <= 57343) {
            const e3 = C.codePointAt(m2 - 1) ?? -1;
            e3 >= 65536 ? (n4 += String.fromCodePoint(e3), m2 += 1, k3 = m2 - 1) : n4 += String.fromCharCode(c2);
            continue;
          }
          n4 += String.fromCharCode(i3);
        }
        else n4 += String.fromCharCode(c2);
        else {
          if (void 0 === C.codePointAt(m2)) continue;
          if (isNewLine(C.codePointAt(m2) ?? -1)) {
            C.codePointAt(m2) === t2 && C.codePointAt(m2 + 1) === r2 && (m2 += 1), m2 += 1, k3 = m2 - 1;
            continue;
          }
          n4 += String.fromCodePoint(consumeEscapedCodePoint());
        }
      }
    }
    function consumeUnicodeRangeToken() {
      m2 += 2, k3 = m2 - 1;
      const e3 = [], n4 = [];
      let t3;
      for (; void 0 !== (t3 = C.codePointAt(m2)) && e3.length < 6 && isHexDigitCodePoint(t3); ) e3.push(t3), m2 += 1, k3 = m2 - 1;
      for (; void 0 !== (t3 = C.codePointAt(m2)) && e3.length < 6 && 63 === t3; ) 0 === n4.length && n4.push(...e3), e3.push(48), n4.push(70), m2 += 1, k3 = m2 - 1;
      if (!n4.length && C.codePointAt(m2) === o2 && isHexDigitCodePoint(C.codePointAt(m2 + 1) ?? -1)) for (m2 += 1, k3 = m2 - 1; void 0 !== (t3 = C.codePointAt(m2)) && n4.length < 6 && isHexDigitCodePoint(t3); ) n4.push(t3), m2 += 1, k3 = m2 - 1;
      if (!n4.length) {
        const n5 = parseInt(String.fromCodePoint(...e3), 16);
        return [u2.UnicodeRange, C.slice(h2, k3 + 1), h2, k3, { startOfRange: n5, endOfRange: n5 }];
      }
      const r3 = parseInt(String.fromCodePoint(...e3), 16), i3 = parseInt(String.fromCodePoint(...n4), 16);
      return [u2.UnicodeRange, C.slice(h2, k3 + 1), h2, k3, { startOfRange: r3, endOfRange: i3 }];
    }
    function consumeUrlToken() {
      for (; isWhitespace(C.codePointAt(m2) ?? -1); ) m2 += 1, k3 = m2 - 1;
      let n4 = "";
      for (; ; ) {
        if (void 0 === C.codePointAt(m2)) {
          const t4 = [u2.URL, C.slice(h2, k3 + 1), h2, k3, { value: n4 }];
          return p(new ParseErrorWithToken(e2.UnexpectedEOFInURL, h2, k3, ["4.3.6. Consume a url token", "Unexpected EOF"], t4)), t4;
        }
        if (41 === C.codePointAt(m2)) return m2 += 1, k3 = m2 - 1, [u2.URL, C.slice(h2, k3 + 1), h2, k3, { value: n4 }];
        if (isWhitespace(C.codePointAt(m2) ?? -1)) {
          for (m2 += 1, k3 = m2 - 1; isWhitespace(C.codePointAt(m2) ?? -1); ) m2 += 1, k3 = m2 - 1;
          if (void 0 === C.codePointAt(m2)) {
            const t4 = [u2.URL, C.slice(h2, k3 + 1), h2, k3, { value: n4 }];
            return p(new ParseErrorWithToken(e2.UnexpectedEOFInURL, h2, k3, ["4.3.6. Consume a url token", "Consume as much whitespace as possible", "Unexpected EOF"], t4)), t4;
          }
          return 41 === C.codePointAt(m2) ? (m2 += 1, k3 = m2 - 1, [u2.URL, C.slice(h2, k3 + 1), h2, k3, { value: n4 }]) : (consumeBadURL(), [u2.BadURL, C.slice(h2, k3 + 1), h2, k3, void 0]);
        }
        const t3 = C.codePointAt(m2);
        if (34 === t3 || 39 === t3 || 40 === t3 || isNonPrintableCodePoint(t3 ?? -1)) {
          consumeBadURL();
          const n5 = [u2.BadURL, C.slice(h2, k3 + 1), h2, k3, void 0];
          return p(new ParseErrorWithToken(e2.UnexpectedCharacterInURL, h2, k3, ["4.3.6. Consume a url token", `Unexpected U+0022 QUOTATION MARK ("), U+0027 APOSTROPHE ('), U+0028 LEFT PARENTHESIS (() or non-printable code point`], n5)), n5;
        }
        if (t3 === s2) {
          if (t3 === s2 && !isNewLine(C.codePointAt(m2 + 1) ?? -1)) {
            m2 += 1, k3 = m2 - 1, n4 += String.fromCodePoint(consumeEscapedCodePoint());
            continue;
          }
          consumeBadURL();
          const o3 = [u2.BadURL, C.slice(h2, k3 + 1), h2, k3, void 0];
          return p(new ParseErrorWithToken(e2.InvalidEscapeSequenceInURL, h2, k3, ["4.3.6. Consume a url token", "U+005C REVERSE SOLIDUS (\\)", "The input stream does not start with a valid escape sequence"], o3)), o3;
        }
        0 === t3 || isSurrogate(t3 ?? -1) ? (n4 += String.fromCharCode(c2), m2 += 1, k3 = m2 - 1) : (n4 += String.fromCodePoint(t3 ?? c2), m2 += +((t3 ?? -1) > 65535) + 1, k3 = m2 - 1);
      }
    }
    function consumeNumericToken() {
      let e3;
      {
        const n5 = C.charCodeAt(m2);
        n5 === o2 ? e3 = "-" : n5 === i2 && (e3 = "+");
      }
      let n4 = a2.Integer;
      const t3 = C.charCodeAt(m2);
      t3 !== i2 && t3 !== o2 || (m2 += 1);
      let r3 = C.charCodeAt(m2);
      for (; r3 >= 48 && r3 <= 57; ) m2 += 1, r3 = C.charCodeAt(m2);
      if (46 === C.charCodeAt(m2) && C.charCodeAt(m2 + 1) >= 48 && C.charCodeAt(m2 + 1) <= 57) for (m2 += 2, n4 = a2.Number, r3 = C.charCodeAt(m2); r3 >= 48 && r3 <= 57; ) m2 += 1, r3 = C.charCodeAt(m2);
      const c3 = C.charCodeAt(m2);
      if (101 === c3 || 69 === c3) {
        const e4 = C.charCodeAt(m2 + 1);
        if (e4 >= 48 && e4 <= 57) for (m2 += 2, n4 = a2.Number, r3 = C.charCodeAt(m2); r3 >= 48 && r3 <= 57; ) m2 += 1, r3 = C.charCodeAt(m2);
        else if ((e4 === o2 || e4 === i2) && C.charCodeAt(m2 + 2) >= 48 && C.charCodeAt(m2 + 2) <= 57) for (m2 += 3, n4 = a2.Number, r3 = C.charCodeAt(m2); r3 >= 48 && r3 <= 57; ) m2 += 1, r3 = C.charCodeAt(m2);
      }
      k3 = m2 - 1;
      const s3 = parseFloat(C.slice(h2, k3 + 1));
      if (checkIfThreeCodePointsWouldStartAnIdentSequence()) {
        const t4 = consumeIdentSequence();
        return [u2.Dimension, C.slice(h2, k3 + 1), h2, k3, { value: s3, signCharacter: e3, type: n4, unit: t4 }];
      }
      return 37 === C.charCodeAt(m2) ? (m2 += 1, k3 = m2 - 1, [u2.Percentage, C.slice(h2, k3 + 1), h2, k3, { value: s3, signCharacter: e3 }]) : [u2.Number, C.slice(h2, k3 + 1), h2, k3, { value: s3, signCharacter: e3, type: n4 }];
    }
    function consumeIdentLikeToken() {
      const e3 = consumeIdentSequence();
      if (40 !== C.charCodeAt(m2)) return [u2.Ident, P2 ? e3 : C.slice(h2, k3 + 1), h2, k3, { value: e3 }];
      if (!(3 !== e3.length || 117 !== e3.charCodeAt(0) && 85 !== e3.charCodeAt(0) || 114 !== e3.charCodeAt(1) && 82 !== e3.charCodeAt(1) || 108 !== e3.charCodeAt(2) && 76 !== e3.charCodeAt(2))) {
        m2 += 1, k3 = m2 - 1;
        let n4 = 0;
        for (; isWhitespace(C.charCodeAt(m2)); ) n4 += 1, m2 += 1, k3 = m2 - 1;
        const t3 = C.charCodeAt(m2);
        return 34 === t3 || 39 === t3 ? (n4 > 0 && (m2 -= n4, k3 = m2 - 1), [u2.Function, C.slice(h2, k3 + 1), h2, k3, { value: e3 }]) : consumeUrlToken();
      }
      return m2 += 1, k3 = m2 - 1, [u2.Function, C.slice(h2, k3 + 1), h2, k3, { value: e3 }];
    }
    function consumeHashToken() {
      m2 += 1, k3 = m2 - 1;
      const e3 = C.charCodeAt(m2);
      if (e3 >= 65 && e3 <= 90 || e3 >= 97 && e3 <= 122 || e3 >= 48 && e3 <= 57 || e3 === o2 || 95 === e3 || isNonASCII_IdentCodePoint(e3) || e3 === s2 && !isNewLine(C.codePointAt(m2 + 1) ?? -1)) {
        let e4 = d.Unrestricted;
        checkIfThreeCodePointsWouldStartAnIdentSequence() && (e4 = d.ID);
        const n4 = consumeIdentSequence();
        return [u2.Hash, C.slice(h2, k3 + 1), h2, k3, { value: n4, type: e4 }];
      }
      return [u2.Delim, "#", h2, k3, { value: "#" }];
    }
    function consumeAtKeywordToken() {
      if (m2 += 1, k3 = m2 - 1, checkIfThreeCodePointsWouldStartAnIdentSequence()) {
        const e3 = consumeIdentSequence();
        return [u2.AtKeyword, C.slice(h2, k3 + 1), h2, k3, { value: e3 }];
      }
      return [u2.Delim, "@", h2, k3, { value: "@" }];
    }
    function consumeInvalidEscapeToken() {
      m2 += 1, k3 = m2 - 1;
      const n4 = [u2.Delim, "\\", h2, k3, { value: "\\" }];
      return p(new ParseErrorWithToken(e2.InvalidEscapeSequenceAfterBackslash, h2, k3, ["4.3.1. Consume a token", "U+005C REVERSE SOLIDUS (\\)", "The input stream does not start with a valid escape sequence"], n4)), n4;
    }
    function checkIfThreeCodePointsWouldStartAnIdentSequence(e3) {
      const n4 = void 0 === e3 ? C.charCodeAt(m2) : e3;
      return n4 === o2 ? C.charCodeAt(m2 + 1) === o2 || (!!isIdentStartCodePoint(C.charCodeAt(m2 + 1)) || C.charCodeAt(m2 + 1) === s2 && !isNewLine(C.charCodeAt(m2 + 2))) : !!isIdentStartCodePoint(n4) || C.charCodeAt(m2) === s2 && !isNewLine(C.charCodeAt(m2 + 1));
    }
    function checkIfThreeCodePointsWouldStartANumber(e3) {
      return e3 === i2 || e3 === o2 ? C.charCodeAt(m2 + 1) >= 48 && C.charCodeAt(m2 + 1) <= 57 || 46 === C.charCodeAt(m2 + 1) && (C.charCodeAt(m2 + 2) >= 48 && C.charCodeAt(m2 + 2) <= 57) : 46 === e3 ? C.charCodeAt(m2 + 1) >= 48 && C.charCodeAt(m2 + 1) <= 57 : e3 >= 48 && e3 <= 57;
    }
    return { nextToken: function nextToken() {
      if (h2 = m2, k3 = -1, m2 >= C.length) return [u2.EOF, "", -1, -1, void 0];
      const e3 = C.charCodeAt(m2);
      if (47 === e3 && 42 === C.charCodeAt(m2 + 1)) return consumeComment2();
      if (f2 && (117 === e3 || 85 === e3) && C.codePointAt(m2 + 1) === i2 && (63 === C.codePointAt(m2 + 2) || isHexDigitCodePoint(C.codePointAt(m2 + 2) ?? -1))) return consumeUnicodeRangeToken();
      if (isIdentStartCodePoint(e3)) return consumeIdentLikeToken();
      if ((n4 = e3) >= 48 && n4 <= 57) return consumeNumericToken();
      switch (e3) {
        case 44:
          return m2 += 1, k3 = m2 - 1, [u2.Comma, ",", h2, k3, void 0];
        case 58:
          return m2 += 1, k3 = m2 - 1, [u2.Colon, ":", h2, k3, void 0];
        case 59:
          return m2 += 1, k3 = m2 - 1, [u2.Semicolon, ";", h2, k3, void 0];
        case 40:
          return m2 += 1, k3 = m2 - 1, [u2.OpenParen, "(", h2, k3, void 0];
        case 41:
          return m2 += 1, k3 = m2 - 1, [u2.CloseParen, ")", h2, k3, void 0];
        case 91:
          return m2 += 1, k3 = m2 - 1, [u2.OpenSquare, "[", h2, k3, void 0];
        case 93:
          return m2 += 1, k3 = m2 - 1, [u2.CloseSquare, "]", h2, k3, void 0];
        case 123:
          return m2 += 1, k3 = m2 - 1, [u2.OpenCurly, "{", h2, k3, void 0];
        case 125:
          return m2 += 1, k3 = m2 - 1, [u2.CloseCurly, "}", h2, k3, void 0];
        case 39:
        case 34:
          return consumeStringToken();
        case 35:
          return consumeHashToken();
        case i2:
        case 46:
          return checkIfThreeCodePointsWouldStartANumber(e3) ? consumeNumericToken() : (m2 += 1, k3 = m2 - 1, [u2.Delim, C[h2], h2, k3, { value: C[h2] }]);
        case r2:
        case t2:
        case 12:
        case 9:
        case 32:
          {
            let e4 = C.charCodeAt(m2);
            for (; 32 === e4 || e4 === r2 || 9 === e4 || e4 === t2 || 12 === e4; ) m2 += 1, e4 = C.charCodeAt(m2);
            k3 = m2 - 1;
          }
          return [u2.Whitespace, C.slice(h2, k3 + 1), h2, k3, void 0];
        case o2:
          return checkIfThreeCodePointsWouldStartANumber(e3) ? consumeNumericToken() : C.charCodeAt(m2 + 1) === o2 && 62 === C.charCodeAt(m2 + 2) ? (m2 += 3, k3 = m2 - 1, [u2.CDC, "-->", h2, k3, void 0]) : checkIfThreeCodePointsWouldStartAnIdentSequence(e3) ? consumeIdentLikeToken() : (m2 += 1, k3 = m2 - 1, [u2.Delim, "-", h2, k3, { value: "-" }]);
        case 60:
          return 33 === C.charCodeAt(m2 + 1) && C.charCodeAt(m2 + 2) === o2 && C.charCodeAt(m2 + 3) === o2 ? (m2 += 4, k3 = m2 - 1, [u2.CDO, "<!--", h2, k3, void 0]) : (m2 += 1, k3 = m2 - 1, [u2.Delim, "<", h2, k3, { value: "<" }]);
        case 64:
          return consumeAtKeywordToken();
        case s2:
          return e3 !== s2 || isNewLine(C.charCodeAt(m2 + 1)) ? consumeInvalidEscapeToken() : consumeIdentLikeToken();
        default:
          return m2 += 1, k3 = m2 - 1, [u2.Delim, C[h2], h2, k3, { value: C[h2] }];
      }
      var n4;
    }, endOfFile: function endOfFile() {
      return m2 >= C.length;
    } };
  }
  function noop() {
  }
  function mutateUnit(e3, n3) {
    const t3 = [];
    for (const e4 of n3) t3.push(e4.codePointAt(0));
    const o3 = serializeIdent(t3);
    101 === o3[0] && insertEscapedCodePoint(o3, 0, o3[0]);
    const r3 = String.fromCodePoint(...o3), i3 = "+" === e3[4].signCharacter ? e3[4].signCharacter : "", c3 = e3[4].value.toString();
    e3[1] = `${i3}${c3}${r3}`, e3[4].unit = n3;
  }
  function serializeIdent(e3) {
    let n3;
    if (0 === e3[0]) e3.splice(0, 1, c2), n3 = 1;
    else if (e3[0] === o2 && e3[1] === o2) n3 = 2;
    else if (e3[0] === o2 && e3[1]) n3 = 2, isIdentStartCodePoint(e3[1]) || (n3 += insertEscapedCodePoint(e3, 1, e3[1]));
    else {
      if (e3[0] === o2 && !e3[1]) return [92, e3[0]];
      isIdentStartCodePoint(e3[0]) ? n3 = 1 : (n3 = 1, n3 += insertEscapedCodePoint(e3, 0, e3[0]));
    }
    for (let t3 = n3; t3 < e3.length; t3++) 0 !== e3[t3] ? isIdentCodePoint(e3[t3]) || (t3 += insertEscapedCharacter(e3, t3, e3[t3])) : (e3.splice(t3, 1, c2), t3++);
    return e3;
  }
  function insertEscapedCharacter(e3, n3, t3) {
    return e3.splice(n3, 1, 92, t3), 1;
  }
  function insertEscapedCodePoint(e3, n3, t3) {
    const o3 = t3.toString(16), r3 = [];
    for (const e4 of o3) r3.push(e4.codePointAt(0));
    return e3.splice(n3, 1, 92, ...r3, 32), 1 + r3.length;
  }
  !(function(e3) {
    e3.Comment = "comment", e3.AtKeyword = "at-keyword-token", e3.BadString = "bad-string-token", e3.BadURL = "bad-url-token", e3.CDC = "CDC-token", e3.CDO = "CDO-token", e3.Colon = "colon-token", e3.Comma = "comma-token", e3.Delim = "delim-token", e3.Dimension = "dimension-token", e3.EOF = "EOF-token", e3.Function = "function-token", e3.Hash = "hash-token", e3.Ident = "ident-token", e3.Number = "number-token", e3.Percentage = "percentage-token", e3.Semicolon = "semicolon-token", e3.String = "string-token", e3.URL = "url-token", e3.Whitespace = "whitespace-token", e3.OpenParen = "(-token", e3.CloseParen = ")-token", e3.OpenSquare = "[-token", e3.CloseSquare = "]-token", e3.OpenCurly = "{-token", e3.CloseCurly = "}-token", e3.UnicodeRange = "unicode-range-token";
  })(u2 || (u2 = {})), (function(e3) {
    e3.Integer = "integer", e3.Number = "number";
  })(a2 || (a2 = {})), (function(e3) {
    e3.Unrestricted = "unrestricted", e3.ID = "id";
  })(d || (d = {}));
  var l2 = Object.values(u2);
  function isToken(e3) {
    return !!Array.isArray(e3) && (!(e3.length < 4) && (!!l2.includes(e3[0]) && ("string" == typeof e3[1] && ("number" == typeof e3[2] && "number" == typeof e3[3]))));
  }
  function isTokenNumeric(e3) {
    if (!e3) return false;
    switch (e3[0]) {
      case u2.Dimension:
      case u2.Number:
      case u2.Percentage:
        return true;
      default:
        return false;
    }
  }
  function isTokenWhiteSpaceOrComment(e3) {
    if (!e3) return false;
    switch (e3[0]) {
      case u2.Whitespace:
      case u2.Comment:
        return true;
      default:
        return false;
    }
  }
  function isTokenColon(e3) {
    return !!e3 && e3[0] === u2.Colon;
  }
  function isTokenComma(e3) {
    return !!e3 && e3[0] === u2.Comma;
  }
  function isTokenComment(e3) {
    return !!e3 && e3[0] === u2.Comment;
  }
  function isTokenDelim(e3) {
    return !!e3 && e3[0] === u2.Delim;
  }
  function isTokenDimension(e3) {
    return !!e3 && e3[0] === u2.Dimension;
  }
  function isTokenEOF(e3) {
    return !!e3 && e3[0] === u2.EOF;
  }
  function isTokenFunction(e3) {
    return !!e3 && e3[0] === u2.Function;
  }
  function isTokenHash(e3) {
    return !!e3 && e3[0] === u2.Hash;
  }
  function isTokenIdent(e3) {
    return !!e3 && e3[0] === u2.Ident;
  }
  function isTokenNumber(e3) {
    return !!e3 && e3[0] === u2.Number;
  }
  function isTokenPercentage(e3) {
    return !!e3 && e3[0] === u2.Percentage;
  }
  function isTokenSemicolon(e3) {
    return !!e3 && e3[0] === u2.Semicolon;
  }
  function isTokenWhitespace(e3) {
    return !!e3 && e3[0] === u2.Whitespace;
  }
  function isTokenOpenParen(e3) {
    return !!e3 && e3[0] === u2.OpenParen;
  }
  function isTokenCloseParen(e3) {
    return !!e3 && e3[0] === u2.CloseParen;
  }
  function isTokenOpenSquare(e3) {
    return !!e3 && e3[0] === u2.OpenSquare;
  }
  function isTokenOpenCurly(e3) {
    return !!e3 && e3[0] === u2.OpenCurly;
  }

  // node_modules/@csstools/css-parser-algorithms/dist/index.mjs
  var k;
  function walkerIndexGenerator(e3) {
    let n3, t3 = e3.slice(), o3 = -1;
    return (e4, s3, i3) => {
      if (e4 === n3 && e4.length === o3 && e4[i3] === s3) {
        o3 = e4.length;
        const n4 = i3 + 1;
        return n4 < e4.length ? n4 : -1;
      }
      let r3 = -1;
      for (let n4 = t3.indexOf(s3); n4 < t3.length && (r3 = e4.indexOf(t3[n4]), -1 === r3 || r3 < i3); n4++) ;
      return -1 === r3 || r3 === i3 && s3 === e4[i3] && (r3++, r3 >= e4.length) ? (n3 = e4, o3 = e4.length, -1) : (t3 = e4.slice(), n3 = e4, o3 = e4.length, r3);
    };
  }
  !(function(e3) {
    e3.Function = "function", e3.SimpleBlock = "simple-block", e3.Whitespace = "whitespace", e3.Comment = "comment", e3.Token = "token";
  })(k || (k = {}));
  var N = 512;
  function consumeComponentValue(e3, n3, t3 = 0, o3 = 0) {
    const s3 = n3[t3];
    if (isTokenOpenParen(s3) || isTokenOpenCurly(s3) || isTokenOpenSquare(s3)) {
      const s4 = consumeSimpleBlock(e3, n3, t3, o3);
      return { advance: s4.advance, node: s4.node };
    }
    if (isTokenFunction(s3)) {
      const s4 = consumeFunction(e3, n3, t3, o3);
      return { advance: s4.advance, node: s4.node };
    }
    if (isTokenWhitespace(s3)) {
      const o4 = consumeWhitespace(e3, n3, t3);
      return { advance: o4.advance, node: o4.node };
    }
    if (isTokenComment(s3)) {
      const o4 = consumeComment(e3, n3, t3);
      return { advance: o4.advance, node: o4.node };
    }
    return { advance: 1, node: new TokenNode(s3) };
  }
  var ContainerNodeBaseClass = class {
    value = [];
    indexOf(e3) {
      return this.value.indexOf(e3);
    }
    at(e3) {
      if ("number" == typeof e3) return e3 < 0 && (e3 = this.value.length + e3), this.value[e3];
    }
    forEach(e3, n3) {
      if (0 === this.value.length) return;
      const t3 = walkerIndexGenerator(this.value);
      let o3 = 0;
      for (; o3 < this.value.length; ) {
        const s3 = this.value[o3];
        let i3;
        if (n3 && (i3 = { ...n3 }), false === e3({ node: s3, parent: this, state: i3 }, o3)) return false;
        if (o3 = t3(this.value, s3, o3), -1 === o3) break;
      }
    }
    walk(e3, n3) {
      0 !== this.value.length && this.forEach((n4, t3) => false !== e3(n4, t3) && ((!("walk" in n4.node) || !("number" == typeof t3 && this.value[t3] === n4.node || this.value.includes(n4.node)) || false !== n4.node.walk(e3, n4.state)) && void 0), n3);
    }
  };
  var FunctionNode = class _FunctionNode extends ContainerNodeBaseClass {
    type = k.Function;
    name;
    endToken;
    constructor(e3, n3, t3) {
      super(), this.name = e3, this.endToken = n3, this.value = t3;
    }
    getName() {
      return this.name[4].value;
    }
    normalize() {
      isTokenEOF(this.endToken) && (this.endToken = [u2.CloseParen, ")", -1, -1, void 0]);
    }
    tokens() {
      return isTokenEOF(this.endToken) ? [this.name, ...this.value.flatMap((e3) => e3.tokens())] : [this.name, ...this.value.flatMap((e3) => e3.tokens()), this.endToken];
    }
    toString() {
      const n3 = this.value.map((n4) => isToken(n4) ? stringify(n4) : n4.toString()).join("");
      return stringify(this.name) + n3 + stringify(this.endToken);
    }
    toJSON() {
      return { type: this.type, name: this.getName(), tokens: this.tokens(), value: this.value.map((e3) => e3.toJSON()) };
    }
    isFunctionNode() {
      return _FunctionNode.isFunctionNode(this);
    }
    static isFunctionNode(e3) {
      return !!e3 && (e3 instanceof _FunctionNode && e3.type === k.Function);
    }
  };
  function consumeFunction(e3, t3, o3, s3) {
    if (s3 >= N) throw new Error("Maximum nesting depth of 512 exceeded, reduce the complexity of your stylesheet.");
    const i3 = [];
    let r3 = o3 + 1;
    for (; ; ) {
      const a3 = t3[r3];
      if (!a3 || isTokenEOF(a3)) return e3.onParseError(new ParseError("Unexpected EOF while consuming a function.", t3[o3][2], t3[t3.length - 1][3], ["5.4.9. Consume a function", "Unexpected EOF"])), { advance: t3.length - o3, node: new FunctionNode(t3[o3], a3, i3) };
      if (isTokenCloseParen(a3)) return { advance: r3 - o3 + 1, node: new FunctionNode(t3[o3], a3, i3) };
      if (isTokenWhiteSpaceOrComment(a3)) {
        const n3 = consumeAllCommentsAndWhitespace(e3, t3, r3);
        r3 += n3.advance;
        for (let e4 = 0; e4 < n3.nodes.length; e4++) i3.push(n3.nodes[e4]);
        continue;
      }
      const c3 = consumeComponentValue(e3, t3, r3, s3 + 1);
      r3 += c3.advance, i3.push(c3.node);
    }
  }
  var SimpleBlockNode = class _SimpleBlockNode extends ContainerNodeBaseClass {
    type = k.SimpleBlock;
    startToken;
    endToken;
    constructor(e3, n3, t3) {
      super(), this.startToken = e3, this.endToken = n3, this.value = t3;
    }
    normalize() {
      if (isTokenEOF(this.endToken)) {
        const e3 = mirrorVariant(this.startToken);
        e3 && (this.endToken = e3);
      }
    }
    tokens() {
      return isTokenEOF(this.endToken) ? [this.startToken, ...this.value.flatMap((e3) => e3.tokens())] : [this.startToken, ...this.value.flatMap((e3) => e3.tokens()), this.endToken];
    }
    toString() {
      const n3 = this.value.map((n4) => isToken(n4) ? stringify(n4) : n4.toString()).join("");
      return stringify(this.startToken) + n3 + stringify(this.endToken);
    }
    toJSON() {
      return { type: this.type, startToken: this.startToken, tokens: this.tokens(), value: this.value.map((e3) => e3.toJSON()) };
    }
    isSimpleBlockNode() {
      return _SimpleBlockNode.isSimpleBlockNode(this);
    }
    static isSimpleBlockNode(e3) {
      return !!e3 && (e3 instanceof _SimpleBlockNode && e3.type === k.SimpleBlock);
    }
  };
  function consumeSimpleBlock(e3, t3, o3, s3) {
    if (s3 >= N) throw new Error("Maximum nesting depth of 512 exceeded, reduce the complexity of your stylesheet.");
    const i3 = mirrorVariantType(t3[o3][0]);
    if (!i3) throw new Error("Failed to parse, a mirror variant must exist for all block open tokens.");
    const r3 = [];
    let a3 = o3 + 1;
    for (; ; ) {
      const c3 = t3[a3];
      if (!c3 || isTokenEOF(c3)) return e3.onParseError(new ParseError("Unexpected EOF while consuming a simple block.", t3[o3][2], t3[t3.length - 1][3], ["5.4.8. Consume a simple block", "Unexpected EOF"])), { advance: t3.length - o3, node: new SimpleBlockNode(t3[o3], c3, r3) };
      if (c3[0] === i3) return { advance: a3 - o3 + 1, node: new SimpleBlockNode(t3[o3], c3, r3) };
      if (isTokenWhiteSpaceOrComment(c3)) {
        const n3 = consumeAllCommentsAndWhitespace(e3, t3, a3);
        a3 += n3.advance;
        for (let e4 = 0; e4 < n3.nodes.length; e4++) r3.push(n3.nodes[e4]);
        continue;
      }
      const u3 = consumeComponentValue(e3, t3, a3, s3 + 1);
      a3 += u3.advance, r3.push(u3.node);
    }
  }
  var WhitespaceNode = class _WhitespaceNode {
    type = k.Whitespace;
    value;
    constructor(e3) {
      this.value = e3;
    }
    tokens() {
      return this.value;
    }
    toString() {
      return stringify(this.value);
    }
    toJSON() {
      return { type: this.type, tokens: this.tokens() };
    }
    isWhitespaceNode() {
      return _WhitespaceNode.isWhitespaceNode(this);
    }
    static isWhitespaceNode(e3) {
      return !!e3 && (e3 instanceof _WhitespaceNode && e3.type === k.Whitespace);
    }
  };
  function consumeWhitespace(e3, n3, t3) {
    let o3 = t3;
    for (; ; ) {
      const e4 = n3[o3];
      if (!isTokenWhitespace(e4)) return { advance: o3 - t3, node: new WhitespaceNode(n3.slice(t3, o3)) };
      o3++;
    }
  }
  var CommentNode = class _CommentNode {
    type = k.Comment;
    value;
    constructor(e3) {
      this.value = e3;
    }
    tokens() {
      return [this.value];
    }
    toString() {
      return stringify(this.value);
    }
    toJSON() {
      return { type: this.type, tokens: this.tokens() };
    }
    isCommentNode() {
      return _CommentNode.isCommentNode(this);
    }
    static isCommentNode(e3) {
      return !!e3 && (e3 instanceof _CommentNode && e3.type === k.Comment);
    }
  };
  function consumeComment(e3, n3, t3) {
    return { advance: 1, node: new CommentNode(n3[t3]) };
  }
  function consumeAllCommentsAndWhitespace(e3, n3, t3) {
    const o3 = [];
    let s3 = t3;
    for (; ; ) {
      if (isTokenWhitespace(n3[s3])) {
        const e4 = consumeWhitespace(0, n3, s3);
        s3 += e4.advance, o3.push(e4.node);
        continue;
      }
      if (!isTokenComment(n3[s3])) return { advance: s3 - t3, nodes: o3 };
      o3.push(new CommentNode(n3[s3])), s3++;
    }
  }
  var TokenNode = class _TokenNode {
    type = k.Token;
    value;
    constructor(e3) {
      this.value = e3;
    }
    tokens() {
      return [this.value];
    }
    toString() {
      return this.value[1];
    }
    toJSON() {
      return { type: this.type, tokens: this.tokens() };
    }
    isTokenNode() {
      return _TokenNode.isTokenNode(this);
    }
    static isTokenNode(e3) {
      return !!e3 && (e3 instanceof _TokenNode && e3.type === k.Token);
    }
  };
  function parseListOfComponentValues(e3, o3) {
    const s3 = { onParseError: o3?.onParseError ?? (() => {
    }) }, i3 = [...e3];
    if (0 === i3.length) return [];
    isTokenEOF(i3[i3.length - 1]) || i3.push([u2.EOF, "", i3[i3.length - 1][2], i3[i3.length - 1][3], void 0]);
    const r3 = [];
    let a3 = 0;
    for (; ; ) {
      if (!i3[a3] || isTokenEOF(i3[a3])) return r3;
      const e4 = consumeComponentValue(s3, i3, a3, 0);
      r3.push(e4.node), a3 += e4.advance;
    }
  }
  function forEach(e3, n3, t3) {
    if (0 === e3.length) return;
    const o3 = walkerIndexGenerator(e3);
    let s3 = 0;
    for (; s3 < e3.length; ) {
      const i3 = e3[s3];
      let r3;
      if (t3 && (r3 = { ...t3 }), false === n3({ node: i3, parent: { value: e3 }, state: r3 }, s3)) return false;
      if (s3 = o3(e3, i3, s3), -1 === s3) break;
    }
  }
  function walk(e3, n3, t3) {
    0 !== e3.length && forEach(e3, (t4, o3) => false !== n3(t4, o3) && ((!("walk" in t4.node) || !("number" == typeof o3 && e3[o3] === t4.node || e3.includes(t4.node)) || false !== t4.node.walk(n3, t4.state)) && void 0), t3);
  }
  function replaceComponentValues(e3, n3) {
    for (let t3 = 0; t3 < e3.length; t3++) {
      walk(e3[t3], (e4, t4) => {
        if ("number" != typeof t4) return;
        const o3 = n3(e4.node);
        o3 && (Array.isArray(o3) ? e4.parent.value.splice(t4, 1, ...o3) : e4.parent.value.splice(t4, 1, o3));
      });
    }
    return e3;
  }
  function isSimpleBlockNode(e3) {
    return SimpleBlockNode.isSimpleBlockNode(e3);
  }
  function isFunctionNode(e3) {
    return FunctionNode.isFunctionNode(e3);
  }
  function isWhitespaceNode(e3) {
    return WhitespaceNode.isWhitespaceNode(e3);
  }
  function isCommentNode(e3) {
    return CommentNode.isCommentNode(e3);
  }
  function isWhiteSpaceOrCommentNode(e3) {
    return isWhitespaceNode(e3) || isCommentNode(e3);
  }
  function isTokenNode(e3) {
    return TokenNode.isTokenNode(e3);
  }
  function sourceIndices(e3) {
    if (Array.isArray(e3)) {
      const n4 = e3[0];
      if (!n4) return [0, 0];
      const t4 = e3[e3.length - 1] || n4;
      return [sourceIndices(n4)[0], sourceIndices(t4)[1]];
    }
    const n3 = e3.tokens(), t3 = n3[0], o3 = n3[n3.length - 1];
    return t3 && o3 ? [t3[2], o3[3]] : [0, 0];
  }

  // node_modules/@csstools/css-calc/dist/index.mjs
  var ParseError2 = class extends Error {
    sourceStart;
    sourceEnd;
    constructor(e3, n3, t3) {
      super(e3), this.name = "ParseError", this.sourceStart = n3, this.sourceEnd = t3;
    }
  };
  var ParseErrorWithComponentValues = class extends ParseError2 {
    componentValues;
    constructor(n3, t3) {
      super(n3, ...sourceIndices(t3)), this.componentValues = t3;
    }
  };
  var S = { UnexpectedAdditionOfDimensionOrPercentageWithNumber: "Unexpected addition of a dimension or percentage with a number.", UnexpectedSubtractionOfDimensionOrPercentageWithNumber: "Unexpected subtraction of a dimension or percentage with a number." };
  var y = /[A-Z]/g;
  function toLowerCaseAZ(e3) {
    return e3.replace(y, (e4) => String.fromCharCode(e4.charCodeAt(0) + 32));
  }
  var I = { cm: "px", in: "px", mm: "px", pc: "px", pt: "px", px: "px", q: "px", deg: "deg", grad: "deg", rad: "deg", turn: "deg", ms: "s", s: "s", hz: "hz", khz: "hz" };
  var x = /* @__PURE__ */ new Map([["cm", (e3) => e3], ["mm", (e3) => 10 * e3], ["q", (e3) => 40 * e3], ["in", (e3) => e3 / 2.54], ["pc", (e3) => e3 / 2.54 * 6], ["pt", (e3) => e3 / 2.54 * 72], ["px", (e3) => e3 / 2.54 * 96]]);
  var M = /* @__PURE__ */ new Map([["deg", (e3) => e3], ["grad", (e3) => e3 / 0.9], ["rad", (e3) => e3 / 180 * Math.PI], ["turn", (e3) => e3 / 360]]);
  var T = /* @__PURE__ */ new Map([["deg", (e3) => 0.9 * e3], ["grad", (e3) => e3], ["rad", (e3) => 0.9 * e3 / 180 * Math.PI], ["turn", (e3) => 0.9 * e3 / 360]]);
  var P = /* @__PURE__ */ new Map([["hz", (e3) => e3], ["khz", (e3) => e3 / 1e3]]);
  var k2 = /* @__PURE__ */ new Map([["cm", (e3) => 2.54 * e3], ["mm", (e3) => 25.4 * e3], ["q", (e3) => 25.4 * e3 * 4], ["in", (e3) => e3], ["pc", (e3) => 6 * e3], ["pt", (e3) => 72 * e3], ["px", (e3) => 96 * e3]]);
  var W = /* @__PURE__ */ new Map([["hz", (e3) => 1e3 * e3], ["khz", (e3) => e3]]);
  var O = /* @__PURE__ */ new Map([["cm", (e3) => e3 / 10], ["mm", (e3) => e3], ["q", (e3) => 4 * e3], ["in", (e3) => e3 / 25.4], ["pc", (e3) => e3 / 25.4 * 6], ["pt", (e3) => e3 / 25.4 * 72], ["px", (e3) => e3 / 25.4 * 96]]);
  var U = /* @__PURE__ */ new Map([["ms", (e3) => e3], ["s", (e3) => e3 / 1e3]]);
  var L = /* @__PURE__ */ new Map([["cm", (e3) => e3 / 6 * 2.54], ["mm", (e3) => e3 / 6 * 25.4], ["q", (e3) => e3 / 6 * 25.4 * 4], ["in", (e3) => e3 / 6], ["pc", (e3) => e3], ["pt", (e3) => e3 / 6 * 72], ["px", (e3) => e3 / 6 * 96]]);
  var $ = /* @__PURE__ */ new Map([["cm", (e3) => e3 / 72 * 2.54], ["mm", (e3) => e3 / 72 * 25.4], ["q", (e3) => e3 / 72 * 25.4 * 4], ["in", (e3) => e3 / 72], ["pc", (e3) => e3 / 72 * 6], ["pt", (e3) => e3], ["px", (e3) => e3 / 72 * 96]]);
  var V = /* @__PURE__ */ new Map([["cm", (e3) => e3 / 96 * 2.54], ["mm", (e3) => e3 / 96 * 25.4], ["q", (e3) => e3 / 96 * 25.4 * 4], ["in", (e3) => e3 / 96], ["pc", (e3) => e3 / 96 * 6], ["pt", (e3) => e3 / 96 * 72], ["px", (e3) => e3]]);
  var Z2 = /* @__PURE__ */ new Map([["cm", (e3) => e3 / 4 / 10], ["mm", (e3) => e3 / 4], ["q", (e3) => e3], ["in", (e3) => e3 / 4 / 25.4], ["pc", (e3) => e3 / 4 / 25.4 * 6], ["pt", (e3) => e3 / 4 / 25.4 * 72], ["px", (e3) => e3 / 4 / 25.4 * 96]]);
  var z = /* @__PURE__ */ new Map([["deg", (e3) => 180 * e3 / Math.PI], ["grad", (e3) => 180 * e3 / Math.PI / 0.9], ["rad", (e3) => e3], ["turn", (e3) => 180 * e3 / Math.PI / 360]]);
  var q = /* @__PURE__ */ new Map([["ms", (e3) => 1e3 * e3], ["s", (e3) => e3]]);
  var R = /* @__PURE__ */ new Map([["deg", (e3) => 360 * e3], ["grad", (e3) => 360 * e3 / 0.9], ["rad", (e3) => 360 * e3 / 180 * Math.PI], ["turn", (e3) => e3]]);
  var G2 = /* @__PURE__ */ new Map([["cm", x], ["mm", O], ["q", Z2], ["in", k2], ["pc", L], ["pt", $], ["px", V], ["ms", U], ["s", q], ["deg", M], ["grad", T], ["rad", z], ["turn", R], ["hz", P], ["khz", W]]);
  function convertUnit(e3, n3) {
    if (!isTokenDimension(e3)) return n3;
    if (!isTokenDimension(n3)) return n3;
    const t3 = toLowerCaseAZ(e3[4].unit), r3 = toLowerCaseAZ(n3[4].unit);
    if (t3 === r3) return n3;
    const u3 = G2.get(r3);
    if (!u3) return n3;
    const a3 = u3.get(t3);
    if (!a3) return n3;
    const o3 = a3(n3[4].value), i3 = [u2.Dimension, "", n3[2], n3[3], { ...n3[4], signCharacter: o3 < 0 ? "-" : void 0, type: Number.isInteger(o3) ? a2.Integer : a2.Number, value: o3 }];
    return mutateUnit(i3, e3[4].unit), i3;
  }
  function toCanonicalUnit(e3) {
    if (!isTokenDimension(e3)) return e3;
    const n3 = toLowerCaseAZ(e3[4].unit), t3 = I[n3];
    if (n3 === t3) return e3;
    const r3 = G2.get(n3);
    if (!r3) return e3;
    const u3 = r3.get(t3);
    if (!u3) return e3;
    const a3 = u3(e3[4].value), o3 = [u2.Dimension, "", e3[2], e3[3], { ...e3[4], signCharacter: a3 < 0 ? "-" : void 0, type: Number.isInteger(a3) ? a2.Integer : a2.Number, value: a3 }];
    return mutateUnit(o3, t3), o3;
  }
  function addition(e3, t3) {
    if (2 !== e3.length) return -1;
    const r3 = e3[0].value;
    let u3 = e3[1].value;
    if (isTokenNumber(r3) && isTokenNumber(u3)) {
      const e4 = r3[4].value + u3[4].value;
      return new TokenNode([u2.Number, e4.toString(), r3[2], u3[3], { value: e4, type: r3[4].type === a2.Integer && u3[4].type === a2.Integer ? a2.Integer : a2.Number }]);
    }
    if (isTokenPercentage(r3) && isTokenPercentage(u3)) {
      const e4 = r3[4].value + u3[4].value;
      return new TokenNode([u2.Percentage, e4.toString() + "%", r3[2], u3[3], { value: e4 }]);
    }
    if (isTokenDimension(r3) && isTokenDimension(u3) && (u3 = convertUnit(r3, u3), toLowerCaseAZ(r3[4].unit) === toLowerCaseAZ(u3[4].unit))) {
      const e4 = r3[4].value + u3[4].value;
      return new TokenNode([u2.Dimension, e4.toString() + r3[4].unit, r3[2], u3[3], { value: e4, type: r3[4].type === a2.Integer && u3[4].type === a2.Integer ? a2.Integer : a2.Number, unit: r3[4].unit }]);
    }
    return (isTokenNumber(r3) && (isTokenDimension(u3) || isTokenPercentage(u3)) || isTokenNumber(u3) && (isTokenDimension(r3) || isTokenPercentage(r3))) && t3.onParseError?.(new ParseErrorWithComponentValues(S.UnexpectedAdditionOfDimensionOrPercentageWithNumber, e3)), -1;
  }
  function division(e3) {
    if (2 !== e3.length) return -1;
    const t3 = e3[0].value, r3 = e3[1].value;
    if (isTokenNumber(t3) && isTokenNumber(r3)) {
      const e4 = t3[4].value / r3[4].value;
      return new TokenNode([u2.Number, e4.toString(), t3[2], r3[3], { value: e4, type: Number.isInteger(e4) ? a2.Integer : a2.Number }]);
    }
    if (isTokenPercentage(t3) && isTokenNumber(r3)) {
      const e4 = t3[4].value / r3[4].value;
      return new TokenNode([u2.Percentage, e4.toString() + "%", t3[2], r3[3], { value: e4 }]);
    }
    if (isTokenDimension(t3) && isTokenNumber(r3)) {
      const e4 = t3[4].value / r3[4].value;
      return new TokenNode([u2.Dimension, e4.toString() + t3[4].unit, t3[2], r3[3], { value: e4, type: Number.isInteger(e4) ? a2.Integer : a2.Number, unit: t3[4].unit }]);
    }
    return -1;
  }
  function isCalculation(e3) {
    return !!e3 && "object" == typeof e3 && "inputs" in e3 && Array.isArray(e3.inputs) && "operation" in e3;
  }
  function solve(e3, n3) {
    if (-1 === e3) return -1;
    const r3 = [{ calculation: e3, inputs: [], index: 0 }];
    let u3 = -1;
    for (; r3.length; ) {
      const e4 = r3[r3.length - 1];
      if (e4.index < e4.calculation.inputs.length) {
        const n4 = e4.calculation.inputs[e4.index];
        if (e4.index++, isTokenNode(n4)) {
          e4.inputs.push(n4);
          continue;
        }
        r3.push({ calculation: n4, inputs: [], index: 0 });
        continue;
      }
      const a3 = e4.calculation.operation(e4.inputs, n3);
      if (r3.pop(), -1 === a3) return -1;
      if (!r3.length) {
        u3 = a3;
        break;
      }
      r3[r3.length - 1].inputs.push(a3);
    }
    return u3;
  }
  function multiplication(e3) {
    if (2 !== e3.length) return -1;
    const t3 = e3[0].value, r3 = e3[1].value;
    if (isTokenNumber(t3) && isTokenNumber(r3)) {
      const e4 = t3[4].value * r3[4].value;
      return new TokenNode([u2.Number, e4.toString(), t3[2], r3[3], { value: e4, type: t3[4].type === a2.Integer && r3[4].type === a2.Integer ? a2.Integer : a2.Number }]);
    }
    if (isTokenPercentage(t3) && isTokenNumber(r3)) {
      const e4 = t3[4].value * r3[4].value;
      return new TokenNode([u2.Percentage, e4.toString() + "%", t3[2], r3[3], { value: e4 }]);
    }
    if (isTokenNumber(t3) && isTokenPercentage(r3)) {
      const e4 = t3[4].value * r3[4].value;
      return new TokenNode([u2.Percentage, e4.toString() + "%", t3[2], r3[3], { value: e4 }]);
    }
    if (isTokenDimension(t3) && isTokenNumber(r3)) {
      const e4 = t3[4].value * r3[4].value;
      return new TokenNode([u2.Dimension, e4.toString() + t3[4].unit, t3[2], r3[3], { value: e4, type: t3[4].type === a2.Integer && r3[4].type === a2.Integer ? a2.Integer : a2.Number, unit: t3[4].unit }]);
    }
    if (isTokenNumber(t3) && isTokenDimension(r3)) {
      const e4 = t3[4].value * r3[4].value;
      return new TokenNode([u2.Dimension, e4.toString() + r3[4].unit, t3[2], r3[3], { value: e4, type: t3[4].type === a2.Integer && r3[4].type === a2.Integer ? a2.Integer : a2.Number, unit: r3[4].unit }]);
    }
    return -1;
  }
  function resolveGlobalsAndConstants(e3, r3) {
    for (let u3 = 0; u3 < e3.length; u3++) {
      const a3 = e3[u3];
      if (!isTokenNode(a3)) continue;
      const o3 = a3.value;
      if (!isTokenIdent(o3)) continue;
      const i3 = toLowerCaseAZ(o3[4].value);
      switch (i3) {
        case "e":
          e3.splice(u3, 1, new TokenNode([u2.Number, Math.E.toString(), o3[2], o3[3], { value: Math.E, type: a2.Number }]));
          break;
        case "pi":
          e3.splice(u3, 1, new TokenNode([u2.Number, Math.PI.toString(), o3[2], o3[3], { value: Math.PI, type: a2.Number }]));
          break;
        case "infinity":
          e3.splice(u3, 1, new TokenNode([u2.Number, "infinity", o3[2], o3[3], { value: 1 / 0, type: a2.Number }]));
          break;
        case "-infinity":
          e3.splice(u3, 1, new TokenNode([u2.Number, "-infinity", o3[2], o3[3], { value: -1 / 0, type: a2.Number }]));
          break;
        case "nan":
          e3.splice(u3, 1, new TokenNode([u2.Number, "NaN", o3[2], o3[3], { value: Number.NaN, type: a2.Number }]));
          break;
        default:
          if (r3.has(i3)) {
            const t3 = r3.get(i3);
            e3.splice(u3, 1, new TokenNode(t3));
          }
      }
    }
    return e3;
  }
  function unary(e3) {
    if (1 !== e3.length) return -1;
    const n3 = e3[0].value;
    return isTokenNumeric(n3) ? e3[0] : -1;
  }
  function resultToCalculation(e3, n3, t3) {
    return isTokenDimension(n3) ? dimensionToCalculation(e3, n3[4].unit, t3) : isTokenPercentage(n3) ? percentageToCalculation(e3, t3) : isTokenNumber(n3) ? numberToCalculation(e3, t3) : -1;
  }
  function dimensionToCalculation(e3, t3, r3) {
    const u3 = e3.tokens();
    return { inputs: [new TokenNode([u2.Dimension, r3.toString() + t3, u3[0][2], u3[u3.length - 1][3], { value: r3, type: Number.isInteger(r3) ? a2.Integer : a2.Number, unit: t3 }])], operation: unary };
  }
  function percentageToCalculation(e3, t3) {
    const r3 = e3.tokens();
    return { inputs: [new TokenNode([u2.Percentage, t3.toString() + "%", r3[0][2], r3[r3.length - 1][3], { value: t3 }])], operation: unary };
  }
  function numberToCalculation(e3, t3) {
    const r3 = e3.tokens();
    return { inputs: [new TokenNode([u2.Number, t3.toString(), r3[0][2], r3[r3.length - 1][3], { value: t3, type: Number.isInteger(t3) ? a2.Integer : a2.Number }])], operation: unary };
  }
  function solveACos(e3, n3) {
    const t3 = n3.value;
    if (!isTokenNumber(t3)) return -1;
    return dimensionToCalculation(e3, "rad", Math.acos(t3[4].value));
  }
  function solveASin(e3, n3) {
    const t3 = n3.value;
    if (!isTokenNumber(t3)) return -1;
    return dimensionToCalculation(e3, "rad", Math.asin(t3[4].value));
  }
  function solveATan(e3, n3) {
    const t3 = n3.value;
    if (!isTokenNumber(t3)) return -1;
    return dimensionToCalculation(e3, "rad", Math.atan(t3[4].value));
  }
  function isDimensionOrNumber(e3) {
    return isTokenDimension(e3) || isTokenNumber(e3);
  }
  function arrayOfSameNumeric(e3) {
    if (0 === e3.length) return true;
    const n3 = e3[0];
    if (!isTokenNumeric(n3)) return false;
    if (1 === e3.length) return true;
    if (isTokenDimension(n3)) {
      const t3 = toLowerCaseAZ(n3[4].unit);
      for (let r3 = 1; r3 < e3.length; r3++) {
        const u3 = e3[r3];
        if (n3[0] !== u3[0]) return false;
        if (t3 !== toLowerCaseAZ(u3[4].unit)) return false;
      }
      return true;
    }
    for (let t3 = 1; t3 < e3.length; t3++) {
      const r3 = e3[t3];
      if (n3[0] !== r3[0]) return false;
    }
    return true;
  }
  function twoOfSameNumeric(e3, n3) {
    return !!isTokenNumeric(e3) && (isTokenDimension(e3) ? e3[0] === n3[0] && toLowerCaseAZ(e3[4].unit) === toLowerCaseAZ(n3[4].unit) : e3[0] === n3[0]);
  }
  function solveATan2(e3, n3, t3) {
    const r3 = n3.value;
    if (!isDimensionOrNumber(r3)) return -1;
    const u3 = convertUnit(r3, t3.value);
    if (!twoOfSameNumeric(r3, u3)) return -1;
    return dimensionToCalculation(e3, "rad", Math.atan2(r3[4].value, u3[4].value));
  }
  function solveAbs(e3, n3, t3) {
    const r3 = n3.value;
    if (!isTokenNumeric(r3)) return -1;
    if (Number.isNaN(r3[4].value)) return resultToCalculation(e3, r3, Number.NaN);
    if (!t3.rawPercentages && isTokenPercentage(r3)) return -1;
    return resultToCalculation(e3, r3, Math.abs(r3[4].value));
  }
  function solveClamp(e3, n3, r3, u3, a3) {
    if (!isTokenNode(n3) || !isTokenNode(r3) || !isTokenNode(u3)) return -1;
    const o3 = n3.value;
    if (!isTokenNumeric(o3)) return -1;
    const i3 = convertUnit(o3, r3.value);
    if (!twoOfSameNumeric(o3, i3)) return -1;
    const l3 = convertUnit(o3, u3.value);
    if (!twoOfSameNumeric(o3, l3)) return -1;
    if (Number.isNaN(o3[4].value) || Number.isNaN(i3[4].value) || Number.isNaN(l3[4].value)) return resultToCalculation(e3, o3, Number.NaN);
    if (!a3.rawPercentages && isTokenPercentage(o3)) return -1;
    return resultToCalculation(e3, o3, Math.max(o3[4].value, Math.min(i3[4].value, l3[4].value)));
  }
  function solveCos(e3, n3) {
    const t3 = n3.value;
    if (!isDimensionOrNumber(t3)) return -1;
    let r3 = t3[4].value;
    if (isTokenDimension(t3)) switch (t3[4].unit.toLowerCase()) {
      case "rad":
        break;
      case "deg":
        r3 = M.get("rad")(t3[4].value);
        break;
      case "grad":
        r3 = T.get("rad")(t3[4].value);
        break;
      case "turn":
        r3 = R.get("rad")(t3[4].value);
        break;
      default:
        return -1;
    }
    return r3 = Math.cos(r3), numberToCalculation(e3, r3);
  }
  function solveExp(e3, n3) {
    const t3 = n3.value;
    if (!isTokenNumber(t3)) return -1;
    return numberToCalculation(e3, Math.exp(t3[4].value));
  }
  function solveHypot(e3, n3, r3) {
    if (!n3.every(isTokenNode)) return -1;
    const u3 = n3[0].value;
    if (!isTokenNumeric(u3)) return -1;
    const a3 = n3.map((e4) => convertUnit(u3, e4.value));
    if (!arrayOfSameNumeric(a3)) return -1;
    if (a3.some((e4) => Number.isNaN(e4[4].value))) return resultToCalculation(e3, u3, Number.NaN);
    if (!r3.rawPercentages && isTokenPercentage(u3)) return -1;
    const o3 = a3.map((e4) => e4[4].value);
    if (o3.some(Number.isNaN)) return resultToCalculation(e3, u3, Number.NaN);
    const i3 = Math.hypot(...o3);
    return resultToCalculation(e3, u3, i3);
  }
  function solveMax(e3, n3, r3) {
    if (!n3.every(isTokenNode)) return -1;
    const u3 = n3[0].value;
    if (!isTokenNumeric(u3)) return -1;
    const a3 = n3.map((e4) => convertUnit(u3, e4.value));
    if (!arrayOfSameNumeric(a3)) return -1;
    if (a3.some((e4) => Number.isNaN(e4[4].value))) return resultToCalculation(e3, u3, Number.NaN);
    if (!r3.rawPercentages && isTokenPercentage(u3)) return -1;
    const o3 = a3.map((e4) => e4[4].value), i3 = Math.max(...o3);
    return resultToCalculation(e3, u3, i3);
  }
  function solveMin(e3, n3, r3) {
    if (!n3.every(isTokenNode)) return -1;
    const u3 = n3[0].value;
    if (!isTokenNumeric(u3)) return -1;
    const a3 = n3.map((e4) => convertUnit(u3, e4.value));
    if (!arrayOfSameNumeric(a3)) return -1;
    if (a3.some((e4) => Number.isNaN(e4[4].value))) return resultToCalculation(e3, u3, Number.NaN);
    if (!r3.rawPercentages && isTokenPercentage(u3)) return -1;
    const o3 = a3.map((e4) => e4[4].value), i3 = Math.min(...o3);
    return resultToCalculation(e3, u3, i3);
  }
  function solveMod(e3, n3, t3) {
    const r3 = n3.value;
    if (!isTokenNumeric(r3)) return -1;
    const u3 = convertUnit(r3, t3.value);
    if (!twoOfSameNumeric(r3, u3)) return -1;
    let a3;
    return a3 = Number.isNaN(r3[4].value) || Number.isNaN(u3[4].value) || 0 === u3[4].value ? Number.NaN : Number.isFinite(r3[4].value) && (Number.isFinite(u3[4].value) || (u3[4].value !== Number.POSITIVE_INFINITY || r3[4].value !== Number.NEGATIVE_INFINITY && !Object.is(0 * r3[4].value, -0)) && (u3[4].value !== Number.NEGATIVE_INFINITY || r3[4].value !== Number.POSITIVE_INFINITY && !Object.is(0 * r3[4].value, 0))) ? Number.isFinite(u3[4].value) ? (r3[4].value % u3[4].value + u3[4].value) % u3[4].value : r3[4].value : Number.NaN, resultToCalculation(e3, r3, a3);
  }
  function solvePow(e3, n3, t3) {
    const r3 = n3.value, u3 = t3.value;
    if (!isTokenNumber(r3)) return -1;
    if (!twoOfSameNumeric(r3, u3)) return -1;
    if (Number.isNaN(r3[4].value) || Number.isNaN(u3[4].value)) return numberToCalculation(e3, Number.NaN);
    return numberToCalculation(e3, Math.pow(r3[4].value, u3[4].value));
  }
  function solveRem(e3, n3, t3) {
    const r3 = n3.value;
    if (!isTokenNumeric(r3)) return -1;
    const u3 = convertUnit(r3, t3.value);
    if (!twoOfSameNumeric(r3, u3)) return -1;
    let a3;
    return a3 = Number.isNaN(r3[4].value) || Number.isNaN(u3[4].value) || 0 === u3[4].value ? Number.NaN : Number.isFinite(r3[4].value) ? Number.isFinite(u3[4].value) ? r3[4].value % u3[4].value : r3[4].value : Number.NaN, resultToCalculation(e3, r3, a3);
  }
  function snapAsBorderWidth(e3, n3, t3) {
    if (!isTokenDimension(n3)) return -1;
    const r3 = t3.devicePixelLength ?? 1, u3 = [u2.Dimension, `${r3}px`, n3[2], n3[3], { value: r3, type: a2.Integer, unit: "px" }], a3 = convertUnit(u3, n3);
    if (!twoOfSameNumeric(a3, u3)) return -1;
    if (Number.isInteger(a3[4].value / r3)) return resultToCalculation(e3, n3, n3[4].value);
    if (a3[4].value > 0) {
      if (a3[4].value < r3) return resultToCalculation(e3, n3, convertUnit(n3, u3)[4].value);
      const t4 = Math.floor(a3[4].value / u3[4].value) * u3[4].value;
      return a3[4].value = t4, resultToCalculation(e3, n3, convertUnit(n3, a3)[4].value);
    }
    if (Math.abs(a3[4].value) < r3) return resultToCalculation(e3, n3, -1 * convertUnit(n3, u3)[4].value);
    const o3 = Math.ceil(a3[4].value / u3[4].value) * u3[4].value;
    return a3[4].value = o3, resultToCalculation(e3, n3, convertUnit(n3, a3)[4].value);
  }
  function solveRound(e3, n3, t3, r3, u3) {
    const a3 = t3.value;
    if (!isTokenNumeric(a3)) return -1;
    if ("line-width" === n3 && !isTokenDimension(a3)) return -1;
    const o3 = convertUnit(a3, r3.value);
    if (!twoOfSameNumeric(a3, o3)) return -1;
    if (Number.isNaN(a3[4].value) || Number.isNaN(o3[4].value)) return resultToCalculation(e3, a3, Number.NaN);
    if (!u3.rawPercentages && isTokenPercentage(a3)) return -1;
    let i3;
    if (0 === o3[4].value) i3 = Number.NaN;
    else if (Number.isFinite(a3[4].value) || Number.isFinite(o3[4].value)) if (!Number.isFinite(a3[4].value) && Number.isFinite(o3[4].value)) i3 = a3[4].value;
    else if (Number.isFinite(a3[4].value) && !Number.isFinite(o3[4].value)) switch (n3) {
      case "down":
        i3 = a3[4].value < 0 ? -1 / 0 : Object.is(-0, 0 * a3[4].value) ? -0 : 0;
        break;
      case "up":
        i3 = a3[4].value > 0 ? 1 / 0 : Object.is(0, 0 * a3[4].value) ? 0 : -0;
        break;
      default:
        i3 = Object.is(0, 0 * a3[4].value) ? 0 : -0;
    }
    else switch (n3) {
      case "down":
        i3 = (o3[4].value > 0 ? Math.floor : Math.ceil)(a3[4].value / o3[4].value) * o3[4].value;
        break;
      case "up":
        i3 = (o3[4].value > 0 ? Math.ceil : Math.floor)(a3[4].value / o3[4].value) * o3[4].value;
        break;
      case "to-zero":
        i3 = Math.trunc(a3[4].value / o3[4].value) * o3[4].value;
        break;
      default: {
        let t4 = Math.floor(a3[4].value / o3[4].value) * o3[4].value, r4 = Math.ceil(a3[4].value / o3[4].value) * o3[4].value;
        if (t4 > r4) {
          const e4 = t4;
          t4 = r4, r4 = e4;
        }
        const l3 = Math.abs(a3[4].value - t4), c3 = Math.abs(a3[4].value - r4);
        if (i3 = "line-width" !== n3 || 0 !== r4 && 0 !== t4 ? l3 === c3 ? r4 : l3 < c3 ? t4 : r4 : 0 !== r4 ? r4 : t4, "line-width" === n3) {
          const n4 = solve(resultToCalculation(e3, a3, i3), u3);
          return -1 === n4 ? -1 : snapAsBorderWidth(e3, n4.value, u3);
        }
        break;
      }
    }
    else i3 = Number.NaN;
    return resultToCalculation(e3, a3, i3);
  }
  function solveSign(e3, n3, t3) {
    const r3 = n3.value;
    if (!isTokenNumeric(r3)) return -1;
    if (Number.isNaN(r3[4].value)) return numberToCalculation(e3, Number.NaN);
    if (!t3.rawPercentages && isTokenPercentage(r3)) return -1;
    return numberToCalculation(e3, Math.sign(r3[4].value));
  }
  function solveSin(e3, n3) {
    const t3 = n3.value;
    if (!isDimensionOrNumber(t3)) return -1;
    let r3 = t3[4].value;
    if (isTokenDimension(t3)) switch (toLowerCaseAZ(t3[4].unit)) {
      case "rad":
        break;
      case "deg":
        r3 = M.get("rad")(t3[4].value);
        break;
      case "grad":
        r3 = T.get("rad")(t3[4].value);
        break;
      case "turn":
        r3 = R.get("rad")(t3[4].value);
        break;
      default:
        return -1;
    }
    return r3 = Math.sin(r3), numberToCalculation(e3, r3);
  }
  function solveSqrt(e3, n3) {
    const t3 = n3.value;
    if (!isTokenNumber(t3)) return -1;
    return numberToCalculation(e3, Math.sqrt(t3[4].value));
  }
  function solveTan(e3, n3) {
    const t3 = n3.value;
    if (!isDimensionOrNumber(t3)) return -1;
    const r3 = t3[4].value;
    let u3 = 0, a3 = t3[4].value;
    if (isTokenDimension(t3)) switch (toLowerCaseAZ(t3[4].unit)) {
      case "rad":
        u3 = z.get("deg")(r3);
        break;
      case "deg":
        u3 = r3, a3 = M.get("rad")(r3);
        break;
      case "grad":
        u3 = T.get("deg")(r3), a3 = T.get("rad")(r3);
        break;
      case "turn":
        u3 = R.get("deg")(r3), a3 = R.get("rad")(r3);
        break;
      default:
        return -1;
    }
    const o3 = u3 / 90;
    return a3 = u3 % 90 == 0 && o3 % 2 != 0 ? (o3 % 4 + 4) % 4 == 1 ? 1 / 0 : -1 / 0 : Math.tan(a3), numberToCalculation(e3, a3);
  }
  function subtraction(e3, t3) {
    if (2 !== e3.length) return -1;
    const r3 = e3[0].value;
    let u3 = e3[1].value;
    if (isTokenNumber(r3) && isTokenNumber(u3)) {
      const e4 = r3[4].value - u3[4].value;
      return new TokenNode([u2.Number, e4.toString(), r3[2], u3[3], { value: e4, type: r3[4].type === a2.Integer && u3[4].type === a2.Integer ? a2.Integer : a2.Number }]);
    }
    if (isTokenPercentage(r3) && isTokenPercentage(u3)) {
      const e4 = r3[4].value - u3[4].value;
      return new TokenNode([u2.Percentage, e4.toString() + "%", r3[2], u3[3], { value: e4 }]);
    }
    if (isTokenDimension(r3) && isTokenDimension(u3) && (u3 = convertUnit(r3, u3), toLowerCaseAZ(r3[4].unit) === toLowerCaseAZ(u3[4].unit))) {
      const e4 = r3[4].value - u3[4].value;
      return new TokenNode([u2.Dimension, e4.toString() + r3[4].unit, r3[2], u3[3], { value: e4, type: r3[4].type === a2.Integer && u3[4].type === a2.Integer ? a2.Integer : a2.Number, unit: r3[4].unit }]);
    }
    return (isTokenNumber(r3) && (isTokenDimension(u3) || isTokenPercentage(u3)) || isTokenNumber(u3) && (isTokenDimension(r3) || isTokenPercentage(r3))) && t3.onParseError?.(new ParseErrorWithComponentValues(S.UnexpectedSubtractionOfDimensionOrPercentageWithNumber, e3)), -1;
  }
  function solveLog(e3, n3) {
    if (1 === n3.length) {
      const r3 = n3[0];
      if (!r3 || !isTokenNode(r3)) return -1;
      const u3 = r3.value;
      if (!isTokenNumber(u3)) return -1;
      return numberToCalculation(e3, Math.log(u3[4].value));
    }
    if (2 === n3.length) {
      const r3 = n3[0];
      if (!r3 || !isTokenNode(r3)) return -1;
      const u3 = r3.value;
      if (!isTokenNumber(u3)) return -1;
      const a3 = n3[1];
      if (!a3 || !isTokenNode(a3)) return -1;
      const o3 = a3.value;
      if (!isTokenNumber(o3)) return -1;
      if (Number.isNaN(u3[4].value) || Number.isNaN(o3[4].value)) return numberToCalculation(e3, Number.NaN);
      if (1 === o3[4].value || o3[4].value <= 0) return numberToCalculation(e3, Number.NaN);
      if (1 === u3[4].value) return numberToCalculation(e3, 0);
      return numberToCalculation(e3, Math.log(u3[4].value) / Math.log(o3[4].value));
    }
    return -1;
  }
  var j = /^none$/i;
  function isNone(e3) {
    if (Array.isArray(e3)) {
      const n4 = e3.filter((e4) => !isWhiteSpaceOrCommentNode(e4));
      return 1 === n4.length && isNone(n4[0]);
    }
    if (!isTokenNode(e3)) return false;
    const n3 = e3.value;
    return !!isTokenIdent(n3) && j.test(n3[4].value);
  }
  function patchPrecision(e3, n3 = 13) {
    if (-1 === e3) return -1;
    if (n3 <= 0) return e3;
    if (isFunctionNode(e3)) return e3;
    const t3 = e3.value;
    if (!isTokenNumeric(t3)) return e3;
    if (shouldSkipPrecisionRounding(t3[4].value)) return e3;
    const r3 = roundToPrecision(t3[4].value, n3).toString();
    return isTokenNumber(t3) ? t3[1] = r3 : isTokenPercentage(t3) ? t3[1] = r3 + "%" : isTokenDimension(t3) && (t3[1] = r3 + t3[4].unit), e3;
  }
  function shouldSkipPrecisionRounding(e3) {
    if (!Number.isFinite(e3) || 0 === e3) return true;
    if (Number.isInteger(e3)) return true;
    const n3 = e3.toString();
    return n3.includes("e") || n3.includes("E");
  }
  function roundToPrecision(e3, n3 = 13) {
    return shouldSkipPrecisionRounding(e3) ? e3 : Number(e3.toFixed(n3));
  }
  var Y2 = String.fromCodePoint(0);
  function solveRandom(e3, n3, t3, r3, u3, a3) {
    if (-1 === n3.fixed && !a3.randomCaching) return -1;
    const o3 = t3.value;
    if (!isTokenNumeric(o3)) return -1;
    const i3 = convertUnit(o3, r3.value);
    if (!twoOfSameNumeric(o3, i3)) return -1;
    let l3 = null;
    if (u3 && (l3 = convertUnit(o3, u3.value), !twoOfSameNumeric(o3, l3))) return -1;
    if (Number.isNaN(o3[4].value) || Number.isNaN(i3[4].value)) return resultToCalculation(e3, o3, Number.NaN);
    if (!Number.isFinite(o3[4].value)) return resultToCalculation(e3, o3, o3[4].value);
    if (!Number.isFinite(i3[4].value) || !Number.isFinite(i3[4].value - o3[4].value)) return resultToCalculation(e3, o3, Number.NaN);
    if (l3) {
      if (Number.isNaN(l3[4].value)) return resultToCalculation(e3, o3, Number.NaN);
      if (!Number.isFinite(l3[4].value)) return resultToCalculation(e3, o3, o3[4].value);
    }
    const c3 = -1 !== n3.fixed ? () => n3.fixed : (a3.randomCaching.propertyN || (a3.randomCaching.propertyN = 0), sfc32(crc32([n3.dashedIdent ? n3.dashedIdent : "", n3.elementScoped ? a3.randomCaching.elementID : "", n3.propertyScoped || n3.propertyIndexScoped ? a3.randomCaching.propertyName : "", n3.propertyIndexScoped ? a3.randomCaching.propertyN : "", a3.randomCaching.documentID].join(Y2)))), s3 = o3[4].value;
    let v = i3[4].value;
    if (v < s3 && (v = s3), l3 && (l3[4].value <= 0 || Math.abs(s3 - v) / l3[4].value > 1e10) && (l3 = null), l3) {
      const n4 = l3[4].value, t4 = Math.max(n4 / 1e3, 1e-9);
      let r4 = Math.floor((v - s3) / n4);
      Math.abs(s3 + r4 * n4 - v) >= t4 && Math.abs(s3 + (r4 + 1) * n4 - v) < t4 && (r4 += 1);
      const u4 = Math.floor(c3() * (r4 + 1));
      let a4 = s3 + u4 * n4;
      return u4 === r4 && Math.abs(a4 - v) < t4 && (a4 = v), resultToCalculation(e3, o3, roundToPrecision(a4, 5));
    }
    return resultToCalculation(e3, o3, roundToPrecision(c3() * (v - s3) + s3, 5));
  }
  function sfc32(e3 = 0.34944106645296036, n3 = 0.19228640875738723, t3 = 0.8784393832007205, r3 = 0.04850964319275053) {
    return () => {
      const u3 = ((e3 |= 0) + (n3 |= 0) | 0) + (r3 |= 0) | 0;
      return r3 = r3 + 1 | 0, e3 = n3 ^ n3 >>> 9, n3 = (t3 |= 0) + (t3 << 3) | 0, t3 = (t3 = t3 << 21 | t3 >>> 11) + u3 | 0, (u3 >>> 0) / 4294967296;
    };
  }
  var _2 = (() => {
    const e3 = new Uint32Array(256);
    for (let n3 = 0; n3 < 256; n3++) e3[n3] = Number("0x" + "00000000 77073096 EE0E612C 990951BA 076DC419 706AF48F E963A535 9E6495A3 0EDB8832 79DCB8A4 E0D5E91E 97D2D988 09B64C2B 7EB17CBD E7B82D07 90BF1D91 1DB71064 6AB020F2 F3B97148 84BE41DE 1ADAD47D 6DDDE4EB F4D4B551 83D385C7 136C9856 646BA8C0 FD62F97A 8A65C9EC 14015C4F 63066CD9 FA0F3D63 8D080DF5 3B6E20C8 4C69105E D56041E4 A2677172 3C03E4D1 4B04D447 D20D85FD A50AB56B 35B5A8FA 42B2986C DBBBC9D6 ACBCF940 32D86CE3 45DF5C75 DCD60DCF ABD13D59 26D930AC 51DE003A C8D75180 BFD06116 21B4F4B5 56B3C423 CFBA9599 B8BDA50F 2802B89E 5F058808 C60CD9B2 B10BE924 2F6F7C87 58684C11 C1611DAB B6662D3D 76DC4190 01DB7106 98D220BC EFD5102A 71B18589 06B6B51F 9FBFE4A5 E8B8D433 7807C9A2 0F00F934 9609A88E E10E9818 7F6A0DBB 086D3D2D 91646C97 E6635C01 6B6B51F4 1C6C6162 856530D8 F262004E 6C0695ED 1B01A57B 8208F4C1 F50FC457 65B0D9C6 12B7E950 8BBEB8EA FCB9887C 62DD1DDF 15DA2D49 8CD37CF3 FBD44C65 4DB26158 3AB551CE A3BC0074 D4BB30E2 4ADFA541 3DD895D7 A4D1C46D D3D6F4FB 4369E96A 346ED9FC AD678846 DA60B8D0 44042D73 33031DE5 AA0A4C5F DD0D7CC9 5005713C 270241AA BE0B1010 C90C2086 5768B525 206F85B3 B966D409 CE61E49F 5EDEF90E 29D9C998 B0D09822 C7D7A8B4 59B33D17 2EB40D81 B7BD5C3B C0BA6CAD EDB88320 9ABFB3B6 03B6E20C 74B1D29A EAD54739 9DD277AF 04DB2615 73DC1683 E3630B12 94643B84 0D6D6A3E 7A6A5AA8 E40ECF0B 9309FF9D 0A00AE27 7D079EB1 F00F9344 8708A3D2 1E01F268 6906C2FE F762575D 806567CB 196C3671 6E6B06E7 FED41B76 89D32BE0 10DA7A5A 67DD4ACC F9B9DF6F 8EBEEFF9 17B7BE43 60B08ED5 D6D6A3E8 A1D1937E 38D8C2C4 4FDFF252 D1BB67F1 A6BC5767 3FB506DD 48B2364B D80D2BDA AF0A1B4C 36034AF6 41047A60 DF60EFC3 A867DF55 316E8EEF 4669BE79 CB61B38C BC66831A 256FD2A0 5268E236 CC0C7795 BB0B4703 220216B9 5505262F C5BA3BBE B2BD0B28 2BB45A92 5CB36A04 C2D7FFA7 B5D0CF31 2CD99E8B 5BDEAE1D 9B64C2B0 EC63F226 756AA39C 026D930A 9C0906A9 EB0E363F 72076785 05005713 95BF4A82 E2B87A14 7BB12BAE 0CB61B38 92D28E9B E5D5BE0D 7CDCEFB7 0BDBDF21 86D3D2D4 F1D4E242 68DDB3F8 1FDA836E 81BE16CD F6B9265B 6FB077E1 18B74777 88085AE6 FF0F6A70 66063BCA 11010B5C 8F659EFF F862AE69 616BFFD3 166CCF45 A00AE278 D70DD2EE 4E048354 3903B3C2 A7672661 D06016F7 4969474D 3E6E77DB AED16A4A D9D65ADC 40DF0B66 37D83BF0 A9BCAE53 DEBB9EC5 47B2CF7F 30B5FFE9 BDBDF21C CABAC28A 53B39330 24B4A3A6 BAD03605 CDD70693 54DE5729 23D967BF B3667A2E C4614AB8 5D681B02 2A6F2B94 B40BBE37 C30C8EA1 5A05DF1B 2D02EF8D".substring(9 * n3, 9 * n3 + 8));
    return e3;
  })();
  function crc32(e3) {
    let n3, t3, r3 = 0;
    r3 ^= -1;
    for (let u3 = 0, a3 = e3.length; u3 < a3; u3++) t3 = 255 & (r3 ^ e3.charCodeAt(u3)), n3 = _2[t3], r3 = r3 >>> 8 ^ n3;
    return (-1 ^ r3) >>> 0;
  }
  function solveCalcMix(e3, n3) {
    const t3 = n3[0].calcSum.value;
    if (!isTokenNumeric(t3)) return -1;
    const r3 = n3.map((e4) => ({ calcSum: convertUnit(t3, e4.calcSum.value), percentage: e4.percentage }));
    if (!arrayOfSameNumeric(r3.map((e4) => e4.calcSum))) return -1;
    const { items: u3 } = normalizeMixPercentages(r3, false), a3 = u3.map((e4) => e4.calcSum[4].value * e4.percentage / 100);
    let o3 = 0;
    for (let e4 = 0; e4 < a3.length; e4++) o3 += a3[e4];
    return resultToCalculation(e3, t3, o3);
  }
  function normalizeMixPercentages(e3, n3 = false) {
    let t3 = 0, r3 = 0;
    for (const n4 of e3) n4.percentage && (t3 += n4.percentage), false === n4.percentage && r3++;
    t3 = Math.min(100, t3);
    for (const n4 of e3) false === n4.percentage && (n4.percentage = (100 - t3) / r3);
    const u3 = e3.slice();
    let a3 = 0;
    for (const e4 of u3) a3 += e4.percentage;
    if (a3 > 100 || a3 > 0 && n3) for (const e4 of u3) e4.percentage = e4.percentage * (100 / a3);
    let o3 = 0;
    return a3 < 100 && (o3 = 100 - a3), { items: u3, leftover: o3 };
  }
  var H = /* @__PURE__ */ new Map([["abs", function abs(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveAbs);
  }], ["acos", function acos(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveACos);
  }], ["asin", function asin(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveASin);
  }], ["atan", function atan(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveATan);
  }], ["atan2", function atan2(e3, n3, t3) {
    return twoCommaSeparatedNodesSolver(e3, n3, t3, solveATan2);
  }], ["calc", calc$1], ["calc-mix", function calcMix(e3, n3, t3) {
    const r3 = variadicArgumentsCalcMix(e3, e3.value, n3, t3);
    if (-1 === r3) return -1;
    return solveCalcMix(e3, r3);
  }], ["clamp", function clamp(u3, a3, i3) {
    const l3 = resolveGlobalsAndConstants([...u3.value.filter((e3) => !isWhiteSpaceOrCommentNode(e3))], a3), c3 = [], s3 = [], f2 = [];
    {
      let e3 = c3;
      for (let n3 = 0; n3 < l3.length; n3++) {
        const r3 = l3[n3];
        if (isTokenNode(r3) && isTokenComma(r3.value)) {
          if (e3 === f2) return -1;
          if (e3 === s3) {
            e3 = f2;
            continue;
          }
          if (e3 === c3) {
            e3 = s3;
            continue;
          }
          return -1;
        }
        e3.push(r3);
      }
    }
    const p = isNone(c3), m2 = isNone(f2);
    if (p && m2) return calc$1(calcWrapper(u3, s3), a3, i3);
    const d2 = solve(calc$1(calcWrapper(u3, s3), a3, i3), i3);
    if (-1 === d2) return -1;
    if (p) {
      const t3 = solve(calc$1(calcWrapper(u3, f2), a3, i3), i3);
      return -1 === t3 ? -1 : solveMin((N2 = u3, C = d2, g2 = t3, new FunctionNode([u2.Function, "min(", N2.name[2], N2.name[3], { value: "min" }], [u2.CloseParen, ")", N2.endToken[2], N2.endToken[3], void 0], [C, new TokenNode([u2.Comma, ",", ...sourceIndices(C), void 0]), g2])), [d2, t3], i3);
    }
    if (m2) {
      const e3 = solve(calc$1(calcWrapper(u3, c3), a3, i3), i3);
      return -1 === e3 ? -1 : solveMax(maxWrapper(u3, e3, d2), [e3, d2], i3);
    }
    var N2, C, g2;
    const h2 = solve(calc$1(calcWrapper(u3, c3), a3, i3), i3);
    if (-1 === h2) return -1;
    const b2 = solve(calc$1(calcWrapper(u3, f2), a3, i3), i3);
    if (-1 === b2) return -1;
    return solveClamp(u3, h2, d2, b2, i3);
  }], ["cos", function cos(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveCos);
  }], ["exp", function exp(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveExp);
  }], ["hypot", function hypot(e3, n3, t3) {
    return variadicNodesSolver(e3, n3, t3, solveHypot);
  }], ["log", function log(e3, n3, t3) {
    return variadicNodesSolver(e3, n3, t3, solveLog);
  }], ["max", function max(e3, n3, t3) {
    return variadicNodesSolver(e3, n3, t3, solveMax);
  }], ["min", function min(e3, n3, t3) {
    return variadicNodesSolver(e3, n3, t3, solveMin);
  }], ["mod", function mod(e3, n3, t3) {
    return twoCommaSeparatedNodesSolver(e3, n3, t3, solveMod);
  }], ["pow", function pow(e3, n3, t3) {
    return twoCommaSeparatedNodesSolver(e3, n3, t3, solvePow);
  }], ["random", function random(e3, n3, t3) {
    const u3 = parseRandomValueSharing(e3, e3.value.filter((e4) => !isWhiteSpaceOrCommentNode(e4)), n3, t3);
    if (-1 === u3) return -1;
    const [a3, o3] = u3, i3 = variadicArguments(e3, o3, n3, t3);
    if (-1 === i3) return -1;
    const [l3, c3, s3] = i3;
    if (!l3 || !c3) return -1;
    return solveRandom(e3, a3, l3, c3, s3, t3);
  }], ["rem", function rem(e3, n3, t3) {
    return twoCommaSeparatedNodesSolver(e3, n3, t3, solveRem);
  }], ["round", function round(e3, u3, a3) {
    const o3 = resolveGlobalsAndConstants([...e3.value.filter((e4) => !isWhiteSpaceOrCommentNode(e4))], u3);
    let i3 = "", l3 = false;
    const c3 = [], p = [];
    {
      let e4 = c3;
      for (let n3 = 0; n3 < o3.length; n3++) {
        const r3 = o3[n3];
        if (!i3 && 0 === c3.length && 0 === p.length && isTokenNode(r3) && isTokenIdent(r3.value)) {
          const e5 = r3.value[4].value.toLowerCase();
          if (K.has(e5)) {
            i3 = e5;
            continue;
          }
        }
        if (isTokenNode(r3) && isTokenComma(r3.value)) {
          if (e4 === p) return -1;
          if (e4 === c3 && i3 && 0 === c3.length) continue;
          if (e4 === c3) {
            l3 = true, e4 = p;
            continue;
          }
          return -1;
        }
        e4.push(r3);
      }
    }
    const m2 = solve(calc$1(calcWrapper(e3, c3), u3, a3), a3);
    if (-1 === m2) return -1;
    if ("line-width" === i3) {
      const e4 = convertUnit([u2.Dimension, "1px", m2.value[2], m2.value[3], { value: 1, type: a2.Integer, unit: "px" }], m2.value);
      if (!isTokenDimension(e4) || "px" !== e4[4].unit) return -1;
    }
    if (!l3 && 0 === p.length) {
      if ("line-width" === i3) return snapAsBorderWidth(e3, m2.value, a3);
      p.push(new TokenNode([u2.Number, "1", m2.value[2], m2.value[3], { value: 1, type: a2.Integer }]));
    }
    const d2 = solve(calc$1(calcWrapper(e3, p), u3, a3), a3);
    if (-1 === d2) return -1;
    i3 || (i3 = "nearest");
    return solveRound(e3, i3, m2, d2, a3);
  }], ["sign", function sign(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveSign);
  }], ["sin", function sin(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveSin);
  }], ["sqrt", function sqrt(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveSqrt);
  }], ["tan", function tan(e3, n3, t3) {
    return singleNodeSolver(e3, n3, t3, solveTan);
  }]]);
  var J = 5e4;
  function calc$1(e3, n3, o3) {
    const i3 = resolveGlobalsAndConstants([...e3.value.filter((e4) => !isWhiteSpaceOrCommentNode(e4))], n3);
    if (i3.length > J) throw new Error(`Maximum number of nodes in a math function (${J}) exceeded, reduce the complexity of your expression`);
    if (1 === i3.length && isTokenNode(i3[0])) return { inputs: [i3[0]], operation: unary };
    let l3 = 0;
    for (; l3 < i3.length; ) {
      const e4 = i3[l3];
      if (isSimpleBlockNode(e4) && isTokenOpenParen(e4.startToken)) {
        const t3 = calc$1(e4, n3, o3);
        if (-1 === t3) return -1;
        i3.splice(l3, 1, t3);
        continue;
      }
      if (isFunctionNode(e4)) {
        const t3 = H.get(e4.getName().toLowerCase());
        if (!t3) return -1;
        const r3 = t3(e4, n3, o3);
        if (-1 === r3) return -1;
        i3.splice(l3, 1, r3);
        continue;
      }
      l3++;
    }
    if (l3 = 0, 1 === i3.length && isCalculation(i3[0])) return i3[0];
    for (; l3 < i3.length; ) {
      const e4 = i3[l3];
      if (!e4 || !isTokenNode(e4) && !isCalculation(e4)) {
        l3++;
        continue;
      }
      const n4 = i3[l3 + 1];
      if (!n4 || !isTokenNode(n4)) {
        l3++;
        continue;
      }
      const r3 = n4.value;
      if (!isTokenDelim(r3) || "*" !== r3[4].value && "/" !== r3[4].value) {
        l3++;
        continue;
      }
      const u3 = i3[l3 + 2];
      if (!u3 || !isTokenNode(u3) && !isCalculation(u3)) return -1;
      "*" !== r3[4].value ? "/" !== r3[4].value ? l3++ : i3.splice(l3, 3, { inputs: [e4, u3], operation: division }) : i3.splice(l3, 3, { inputs: [e4, u3], operation: multiplication });
    }
    if (l3 = 0, 1 === i3.length && isCalculation(i3[0])) return i3[0];
    for (; l3 < i3.length; ) {
      const e4 = i3[l3];
      if (!e4 || !isTokenNode(e4) && !isCalculation(e4)) {
        l3++;
        continue;
      }
      const n4 = i3[l3 + 1];
      if (!n4 || !isTokenNode(n4)) {
        l3++;
        continue;
      }
      const r3 = n4.value;
      if (!isTokenDelim(r3) || "+" !== r3[4].value && "-" !== r3[4].value) {
        l3++;
        continue;
      }
      const u3 = i3[l3 + 2];
      if (!u3 || !isTokenNode(u3) && !isCalculation(u3)) return -1;
      "+" !== r3[4].value ? "-" !== r3[4].value ? l3++ : i3.splice(l3, 3, { inputs: [e4, u3], operation: subtraction }) : i3.splice(l3, 3, { inputs: [e4, u3], operation: addition });
    }
    return 1 === i3.length && isCalculation(i3[0]) ? i3[0] : -1;
  }
  function singleNodeSolver(e3, n3, t3, r3) {
    const u3 = singleArgument(e3, n3, t3);
    return -1 === u3 ? -1 : r3(e3, u3, t3);
  }
  function singleArgument(e3, n3, t3) {
    const u3 = resolveGlobalsAndConstants([...e3.value.filter((e4) => !isWhiteSpaceOrCommentNode(e4))], n3), a3 = solve(calc$1(calcWrapper(e3, u3), n3, t3), t3);
    return -1 === a3 ? -1 : a3;
  }
  function twoCommaSeparatedNodesSolver(e3, n3, t3, r3) {
    const u3 = twoCommaSeparatedArguments(e3, n3, t3);
    if (-1 === u3) return -1;
    const [a3, o3] = u3;
    return r3(e3, a3, o3, t3);
  }
  function twoCommaSeparatedArguments(e3, n3, u3) {
    const a3 = resolveGlobalsAndConstants([...e3.value.filter((e4) => !isWhiteSpaceOrCommentNode(e4))], n3), o3 = [], i3 = [];
    {
      let e4 = o3;
      for (let n4 = 0; n4 < a3.length; n4++) {
        const r3 = a3[n4];
        if (isTokenNode(r3) && isTokenComma(r3.value)) {
          if (e4 === i3) return -1;
          if (e4 === o3) {
            e4 = i3;
            continue;
          }
          return -1;
        }
        e4.push(r3);
      }
    }
    const l3 = solve(calc$1(calcWrapper(e3, o3), n3, u3), u3);
    if (-1 === l3) return -1;
    const c3 = solve(calc$1(calcWrapper(e3, i3), n3, u3), u3);
    return -1 === c3 ? -1 : [l3, c3];
  }
  function variadicNodesSolver(e3, n3, t3, r3) {
    const u3 = variadicArguments(e3, e3.value, n3, t3);
    return -1 === u3 ? -1 : r3(e3, u3, t3);
  }
  function variadicArguments(e3, n3, u3, a3) {
    const o3 = resolveGlobalsAndConstants([...n3.filter((e4) => !isWhiteSpaceOrCommentNode(e4))], u3), i3 = [];
    {
      const n4 = [];
      let r3 = [];
      for (let e4 = 0; e4 < o3.length; e4++) {
        const u4 = o3[e4];
        isTokenNode(u4) && isTokenComma(u4.value) ? (n4.push(r3), r3 = []) : r3.push(u4);
      }
      n4.push(r3);
      for (let t3 = 0; t3 < n4.length; t3++) {
        if (0 === n4[t3].length) return -1;
        const r4 = solve(calc$1(calcWrapper(e3, n4[t3]), u3, a3), a3);
        if (-1 === r4) return -1;
        i3.push(r4);
      }
    }
    return i3;
  }
  function variadicArgumentsCalcMix(e3, n3, u3, a3) {
    const o3 = resolveGlobalsAndConstants([...n3.filter((e4) => !isWhiteSpaceOrCommentNode(e4))], u3), i3 = [];
    {
      const n4 = [];
      let l3 = [];
      for (let e4 = 0; e4 < o3.length; e4++) {
        const r3 = o3[e4];
        isTokenNode(r3) && isTokenComma(r3.value) ? (n4.push(l3), l3 = []) : l3.push(r3);
      }
      n4.push(l3);
      for (let o4 = 0; o4 < n4.length; o4++) {
        if (0 === n4[o4].length) return -1;
        let l4 = -1, c3 = false;
        for (let i4 = n4[o4].length - 1; i4 >= 0; i4--) {
          if (isWhiteSpaceOrCommentNode(n4[o4][i4])) continue;
          if (l4 = solve(calc$1(calcWrapper(e3, n4[o4].slice(0, i4 + 1)), u3, a3), a3), -1 === l4) continue;
          const s3 = n4[o4].slice(i4 + 1).filter((e4) => !isWhiteSpaceOrCommentNode(e4));
          if (s3.length) if (1 === s3.length && isTokenNode(s3[0]) && isTokenPercentage(s3[0].value)) {
            if (c3 = s3[0].value[4].value, c3 < 0 || c3 > 100) return -1;
          } else {
            const n5 = solve(calc$1(calcWrapper(e3, s3), u3, a3), a3);
            if (-1 === n5) return -1;
            if (!isTokenPercentage(n5.value)) return -1;
            c3 = n5.value[4].value, c3 = Math.min(100, Math.max(0, c3));
          }
          break;
        }
        if (-1 === l4) return -1;
        i3.push({ calcSum: l4, percentage: c3 });
      }
    }
    return i3;
  }
  var K = /* @__PURE__ */ new Set(["nearest", "line-width", "up", "down", "to-zero"]);
  function parseRandomValueSharing(e3, n3, r3, u3) {
    const a3 = { dashedIdent: "", fixed: -1, elementScoped: false, propertyScoped: false, propertyIndexScoped: false };
    let o3 = false;
    const i3 = n3[0];
    if (!isTokenNode(i3) || !isTokenIdent(i3.value)) return [a3, n3];
    for (let i4 = 0; i4 < n3.length; i4++) {
      const l3 = n3[i4];
      if (!isTokenNode(l3)) return -1;
      if (isTokenComma(l3.value)) return [a3, n3.slice(i4 + 1)];
      if (!isTokenIdent(l3.value)) return -1;
      const c3 = l3.value[4].value.toLowerCase();
      if ("element-scoped" !== c3) if ("property-scoped" !== c3) if ("property-index-scoped" !== c3) {
        if ("fixed" === c3) {
          if (-1 !== a3.fixed || o3 || a3.dashedIdent || a3.elementScoped || a3.propertyScoped || a3.propertyIndexScoped) return -1;
          i4++;
          const t3 = n3[i4];
          if (!t3) return -1;
          const l4 = solve(calc$1(calcWrapper(e3, [t3]), r3, u3), u3);
          if (-1 === l4) return -1;
          if (!isTokenNumber(l4.value)) return -1;
          if (l4.value[4].value < 0 || l4.value[4].value > 1) return -1;
          a3.fixed = Math.max(0, Math.min(l4.value[4].value, 1 - Number.EPSILON / 2));
          continue;
        }
        if ("auto" !== c3) {
          if (!c3.startsWith("--")) return 0 === i4 ? [a3, n3] : -1;
          if (-1 !== a3.fixed || o3 || a3.dashedIdent) return -1;
          a3.dashedIdent = c3;
        } else {
          if (-1 !== a3.fixed || o3 || a3.dashedIdent || a3.elementScoped || a3.propertyScoped || a3.propertyIndexScoped) return -1;
          a3.elementScoped = true, a3.propertyIndexScoped = true, o3 = true;
        }
      } else {
        if (-1 !== a3.fixed || o3 || a3.propertyScoped || a3.propertyIndexScoped) return -1;
        a3.propertyIndexScoped = true;
      }
      else {
        if (-1 !== a3.fixed || o3 || a3.propertyScoped || a3.propertyIndexScoped) return -1;
        a3.propertyScoped = true;
      }
      else {
        if (-1 !== a3.fixed || o3 || a3.elementScoped) return -1;
        a3.elementScoped = true;
      }
    }
    return -1;
  }
  function calcWrapper(e3, n3) {
    return new FunctionNode([u2.Function, "calc(", e3.name[2], e3.name[3], { value: "calc" }], [u2.CloseParen, ")", e3.endToken[2], e3.endToken[3], void 0], n3);
  }
  function maxWrapper(t3, r3, u3) {
    return new FunctionNode([u2.Function, "max(", t3.name[2], t3.name[3], { value: "max" }], [u2.CloseParen, ")", t3.endToken[2], t3.endToken[3], void 0], [r3, new TokenNode([u2.Comma, ",", ...sourceIndices(r3), void 0]), u3]);
  }
  function patchNaN(e3) {
    if (-1 === e3) return -1;
    if (isFunctionNode(e3)) return e3;
    const t3 = e3.value;
    return isTokenNumeric(t3) && Number.isNaN(t3[4].value) ? isTokenNumber(t3) ? new FunctionNode([u2.Function, "calc(", t3[2], t3[3], { value: "calc" }], [u2.CloseParen, ")", t3[2], t3[3], void 0], [new TokenNode([u2.Ident, "NaN", t3[2], t3[3], { value: "NaN" }])]) : isTokenDimension(t3) ? new FunctionNode([u2.Function, "calc(", t3[2], t3[3], { value: "calc" }], [u2.CloseParen, ")", t3[2], t3[3], void 0], [new TokenNode([u2.Ident, "NaN", t3[2], t3[3], { value: "NaN" }]), new WhitespaceNode([[u2.Whitespace, " ", t3[2], t3[3], void 0]]), new TokenNode([u2.Delim, "*", t3[2], t3[3], { value: "*" }]), new WhitespaceNode([[u2.Whitespace, " ", t3[2], t3[3], void 0]]), new TokenNode([u2.Dimension, "1" + t3[4].unit, t3[2], t3[3], { value: 1, type: a2.Integer, unit: t3[4].unit }])]) : isTokenPercentage(t3) ? new FunctionNode([u2.Function, "calc(", t3[2], t3[3], { value: "calc" }], [u2.CloseParen, ")", t3[2], t3[3], void 0], [new TokenNode([u2.Ident, "NaN", t3[2], t3[3], { value: "NaN" }]), new WhitespaceNode([[u2.Whitespace, " ", t3[2], t3[3], void 0]]), new TokenNode([u2.Delim, "*", t3[2], t3[3], { value: "*" }]), new WhitespaceNode([[u2.Whitespace, " ", t3[2], t3[3], void 0]]), new TokenNode([u2.Percentage, "1%", t3[2], t3[3], { value: 1 }])]) : -1 : e3;
  }
  function patchInfinity(e3) {
    if (-1 === e3) return -1;
    if (isFunctionNode(e3)) return e3;
    const t3 = e3.value;
    if (!isTokenNumeric(t3)) return e3;
    if (Number.isFinite(t3[4].value) || Number.isNaN(t3[4].value)) return e3;
    let r3 = "";
    return Number.NEGATIVE_INFINITY === t3[4].value && (r3 = "-"), isTokenNumber(t3) ? new FunctionNode([u2.Function, "calc(", t3[2], t3[3], { value: "calc" }], [u2.CloseParen, ")", t3[2], t3[3], void 0], [new TokenNode([u2.Ident, r3 + "infinity", t3[2], t3[3], { value: r3 + "infinity" }])]) : isTokenDimension(t3) ? new FunctionNode([u2.Function, "calc(", t3[2], t3[3], { value: "calc" }], [u2.CloseParen, ")", t3[2], t3[3], void 0], [new TokenNode([u2.Ident, r3 + "infinity", t3[2], t3[3], { value: r3 + "infinity" }]), new WhitespaceNode([[u2.Whitespace, " ", t3[2], t3[3], void 0]]), new TokenNode([u2.Delim, "*", t3[2], t3[3], { value: "*" }]), new WhitespaceNode([[u2.Whitespace, " ", t3[2], t3[3], void 0]]), new TokenNode([u2.Dimension, "1" + t3[4].unit, t3[2], t3[3], { value: 1, type: a2.Integer, unit: t3[4].unit }])]) : new FunctionNode([u2.Function, "calc(", t3[2], t3[3], { value: "calc" }], [u2.CloseParen, ")", t3[2], t3[3], void 0], [new TokenNode([u2.Ident, r3 + "infinity", t3[2], t3[3], { value: r3 + "infinity" }]), new WhitespaceNode([[u2.Whitespace, " ", t3[2], t3[3], void 0]]), new TokenNode([u2.Delim, "*", t3[2], t3[3], { value: "*" }]), new WhitespaceNode([[u2.Whitespace, " ", t3[2], t3[3], void 0]]), new TokenNode([u2.Percentage, "1%", t3[2], t3[3], { value: 1 }])]);
  }
  function patchMinusZero(e3) {
    if (-1 === e3) return -1;
    if (isFunctionNode(e3)) return e3;
    const n3 = e3.value;
    return isTokenNumeric(n3) && Object.is(-0, n3[4].value) ? ("-0" === n3[1] || (isTokenPercentage(n3) ? n3[1] = "-0%" : isTokenDimension(n3) ? n3[1] = "-0" + n3[4].unit : n3[1] = "-0"), e3) : e3;
  }
  function patchCanonicalUnit(e3) {
    return -1 === e3 ? -1 : isFunctionNode(e3) ? e3 : isTokenDimension(e3.value) ? (e3.value = toCanonicalUnit(e3.value), e3) : e3;
  }
  function patchCalcResult(e3, n3) {
    let t3 = e3;
    return n3?.toCanonicalUnits && (t3 = patchCanonicalUnit(t3)), t3 = patchPrecision(t3, n3?.precision), t3 = patchMinusZero(t3), n3?.censorIntoStandardRepresentableValues || (t3 = patchNaN(t3), t3 = patchInfinity(t3)), t3;
  }
  function tokenizeGlobals(e3) {
    const n3 = /* @__PURE__ */ new Map();
    if (!e3) return n3;
    for (const [t3, r3] of e3) if (isToken(r3)) n3.set(t3, r3);
    else if ("string" == typeof r3) {
      const e4 = tokenizer({ css: r3 }), u3 = e4.nextToken();
      if (e4.nextToken(), !e4.endOfFile()) continue;
      if (!isTokenNumeric(u3)) continue;
      n3.set(t3, u3);
      continue;
    }
    return n3;
  }
  function calcFromComponentValues(e3, n3) {
    const t3 = tokenizeGlobals(n3?.globals);
    return replaceComponentValues2(e3, (e4) => {
      if (!isFunctionNode(e4)) return;
      const r3 = H.get(e4.getName().toLowerCase());
      if (!r3) return;
      const a3 = patchCalcResult(solve(r3(e4, t3, n3 ?? {}), n3 ?? {}), n3);
      return -1 !== a3 ? n3?.calcWrapper ? calcWrapper(e4, [a3]) : a3 : void 0;
    });
  }
  function replaceComponentValues2(n3, u3) {
    for (let a3 = 0; a3 < n3.length; a3++) {
      const o3 = n3[a3];
      walk(o3, (n4, a4) => {
        if ("number" != typeof a4) return;
        const o4 = u3(n4.node);
        if (!o4) return;
        const l3 = [o4], c3 = n4.parent.value[a4 - 1];
        isTokenNode(c3) && isTokenDelim(c3.value) && ("-" === c3.value[4].value || "+" === c3.value[4].value) && l3.splice(0, 0, new WhitespaceNode([[u2.Whitespace, " ", ...sourceIndices(n4.node), void 0]]));
        const s3 = n4.parent.value[a4 + 1];
        !s3 || isWhiteSpaceOrCommentNode(s3) || isTokenNode(s3) && (isTokenComma(s3.value) || isTokenColon(s3.value) || isTokenSemicolon(s3.value) || isTokenDelim(s3.value) && "-" !== s3.value[4].value && "+" !== s3.value[4].value) || l3.push(new WhitespaceNode([[u2.Whitespace, " ", ...sourceIndices(n4.node), void 0]])), n4.parent.value.splice(a4, 1, ...l3);
      });
    }
    return n3;
  }
  var Q = new Set(H.keys());

  // node_modules/@csstools/css-color-parser/dist/index.mjs
  var xe;
  var De;
  function normalize(e3, n3, a3, o3) {
    return Math.min(Math.max(e3 / n3, a3), o3);
  }
  function convertNaNToZero(e3) {
    return [Number.isNaN(e3[0]) ? 0 : e3[0], Number.isNaN(e3[1]) ? 0 : e3[1], Number.isNaN(e3[2]) ? 0 : e3[2]];
  }
  function colorData_to_XYZ_D50(e3) {
    switch (e3.colorNotation) {
      case xe.HEX:
      case xe.RGB:
      case xe.sRGB:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: sRGB_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.Linear_sRGB:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: lin_sRGB_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.Display_P3:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: P3_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.Linear_Display_P3:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: lin_P3_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.Rec2020:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: rec_2020_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.A98_RGB:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: a98_RGB_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.ProPhoto_RGB:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: ProPhoto_RGB_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.HSL:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: HSL_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.HWB:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: HWB_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.Lab:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: Lab_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.OKLab:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: OKLab_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.LCH:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: LCH_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.OKLCH:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: OKLCH_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.XYZ_D50:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: XYZ_D50_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      case xe.XYZ_D65:
        return { ...e3, colorNotation: xe.XYZ_D50, channels: XYZ_D65_to_XYZ_D50(convertNaNToZero(e3.channels)) };
      default:
        throw new Error("Unsupported color notation");
    }
  }
  function colorData_to_XYZ_D65(e3) {
    switch (e3.colorNotation) {
      case xe.HEX:
      case xe.RGB:
      case xe.sRGB:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: sRGB_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.Linear_sRGB:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: lin_sRGB_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.Display_P3:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: P3_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.Linear_Display_P3:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: lin_P3_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.Rec2020:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: rec_2020_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.A98_RGB:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: a98_RGB_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.ProPhoto_RGB:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: ProPhoto_RGB_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.HSL:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: HSL_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.HWB:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: HWB_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.Lab:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: Lab_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.OKLab:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: OKLab_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.LCH:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: LCH_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.OKLCH:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: OKLCH_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.XYZ_D50:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: XYZ_D50_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      case xe.XYZ_D65:
        return { ...e3, colorNotation: xe.XYZ_D65, channels: XYZ_D65_to_XYZ_D65(convertNaNToZero(e3.channels)) };
      default:
        throw new Error("Unsupported color notation");
    }
  }
  !(function(e3) {
    e3.A98_RGB = "a98-rgb", e3.Display_P3 = "display-p3", e3.Linear_Display_P3 = "display-p3-linear", e3.HEX = "hex", e3.HSL = "hsl", e3.HWB = "hwb", e3.LCH = "lch", e3.Lab = "lab", e3.Linear_sRGB = "srgb-linear", e3.OKLCH = "oklch", e3.OKLab = "oklab", e3.ProPhoto_RGB = "prophoto-rgb", e3.RGB = "rgb", e3.sRGB = "srgb", e3.Rec2020 = "rec2020", e3.XYZ_D50 = "xyz-d50", e3.XYZ_D65 = "xyz-d65";
  })(xe || (xe = {})), (function(e3) {
    e3.ColorKeyword = "color-keyword", e3.HasAlpha = "has-alpha", e3.HasDimensionValues = "has-dimension-values", e3.HasNoneKeywords = "has-none-keywords", e3.HasNumberValues = "has-number-values", e3.HasPercentageAlpha = "has-percentage-alpha", e3.HasPercentageValues = "has-percentage-values", e3.HasVariableAlpha = "has-variable-alpha", e3.Hex = "hex", e3.LegacyHSL = "legacy-hsl", e3.LegacyRGB = "legacy-rgb", e3.NamedColor = "named-color", e3.RelativeColorSyntax = "relative-color-syntax", e3.ColorMix = "color-mix", e3.ColorMixVariadic = "color-mix-variadic", e3.ContrastColor = "contrast-color", e3.RelativeAlphaSyntax = "relative-alpha-syntax", e3.Experimental = "experimental";
  })(De || (De = {}));
  var Le = /* @__PURE__ */ new Set([xe.A98_RGB, xe.Display_P3, xe.Linear_Display_P3, xe.HEX, xe.Linear_sRGB, xe.ProPhoto_RGB, xe.RGB, xe.sRGB, xe.Rec2020, xe.XYZ_D50, xe.XYZ_D65]);
  function colorDataToForInterpolation(e3, n3) {
    if (e3.colorNotation === n3) return { ...e3 };
    const a3 = [...e3.channels];
    let o3 = { ...e3 };
    return o3.channels = convertPowerlessComponentsToMissingComponents(o3.channels, e3.colorNotation), o3 = convertToDestination(o3, n3), o3.channels = carryForwardMissingComponentsForColorAndNotation(o3, e3.colorNotation, n3, a3), o3;
  }
  function colorDataToForRelativeColorSyntax(e3, n3) {
    if (e3.colorNotation === n3) return { ...e3 };
    const a3 = [...e3.channels];
    let o3 = { ...e3 };
    return o3.channels = convertPowerlessComponentsToMissingComponents(o3.channels, e3.colorNotation), o3 = convertToDestination(o3, n3), o3.channels = carryForwardMissingComponentsForColorAndNotation(o3, e3.colorNotation, n3, a3), o3;
  }
  function convertToDestination(e3, n3) {
    if (e3.colorNotation !== n3) {
      const direct = directSRGBConversion(e3, n3);
      if (direct) {
        e3.colorNotation = n3;
        e3.channels = convertPowerlessComponentsToMissingComponents(direct, n3);
        return e3;
      }
    }
    if (e3.colorNotation === n3) return e3.channels = convertNaNToZero(e3.channels), e3.channels = convertPowerlessComponentsToMissingComponents(e3.channels, n3), e3;
    switch (n3) {
      case xe.HEX:
      case xe.RGB: {
        const n4 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.RGB, e3.channels = XYZ_D65_to_sRGB(n4.channels), e3.channels = e3.channels.map((e4) => reducePrecisionOrNaN(e4, 8)), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.sRGB: {
        const n4 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.sRGB, e3.channels = XYZ_D65_to_sRGB(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.Linear_sRGB: {
        const n4 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.Linear_sRGB, e3.channels = XYZ_D65_to_lin_sRGB(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.Display_P3: {
        const n4 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.Display_P3, e3.channels = XYZ_D65_to_P3(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.Linear_Display_P3: {
        const n4 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.Linear_Display_P3, e3.channels = XYZ_D65_to_lin_P3(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.Rec2020: {
        const n4 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.Rec2020, e3.channels = XYZ_D65_to_rec_2020(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.ProPhoto_RGB: {
        const n4 = colorData_to_XYZ_D50(e3);
        e3.colorNotation = xe.ProPhoto_RGB, e3.channels = XYZ_D50_to_ProPhoto(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.A98_RGB: {
        const n4 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.A98_RGB, e3.channels = XYZ_D65_to_a98_RGB(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.HSL: {
        const a3 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.HSL, e3.channels = XYZ_D65_to_HSL(a3.channels), e3.channels = e3.channels.map((e4) => reducePrecisionOrNaN(e4, 8)), e3.channels = normalizeAfterColorConversion(e3.channels, 0), e3.channels = convertPowerlessComponentsToMissingComponents(e3.channels, n3);
        break;
      }
      case xe.HWB: {
        const a3 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.HWB, e3.channels = XYZ_D65_to_HWB(a3.channels), e3.channels = e3.channels.map((e4) => reducePrecisionOrNaN(e4, 8)), e3.channels = normalizeAfterColorConversion(e3.channels, 0), e3.channels = convertPowerlessComponentsToMissingComponents(e3.channels, n3);
        break;
      }
      case xe.Lab: {
        const n4 = colorData_to_XYZ_D50(e3);
        e3.colorNotation = xe.Lab, e3.channels = XYZ_D50_to_Lab(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.LCH: {
        const a3 = colorData_to_XYZ_D50(e3);
        e3.colorNotation = xe.LCH, e3.channels = XYZ_D50_to_LCH(a3.channels), e3.channels = normalizeAfterColorConversion(e3.channels, 2), e3.channels = convertPowerlessComponentsToMissingComponents(e3.channels, n3);
        break;
      }
      case xe.OKLCH: {
        const a3 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.OKLCH, e3.channels = XYZ_D65_to_OKLCH(a3.channels), e3.channels = normalizeAfterColorConversion(e3.channels, 2), e3.channels = convertPowerlessComponentsToMissingComponents(e3.channels, n3);
        break;
      }
      case xe.OKLab: {
        const n4 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.OKLab, e3.channels = XYZ_D65_to_OKLab(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.XYZ_D50: {
        const n4 = colorData_to_XYZ_D50(e3);
        e3.colorNotation = xe.XYZ_D50, e3.channels = XYZ_D50_to_XYZ_D50(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      case xe.XYZ_D65: {
        const n4 = colorData_to_XYZ_D65(e3);
        e3.colorNotation = xe.XYZ_D65, e3.channels = XYZ_D65_to_XYZ_D65(n4.channels), e3.channels = normalizeAfterColorConversion(e3.channels);
        break;
      }
      default:
        throw new Error("Unsupported color notation");
    }
    return e3;
  }
  function carryForwardMissingComponentsForColorAndNotation(e3, n3, a3, o3) {
    if (n3 === a3) return carryForwardMissingComponents(o3, [0, 1, 2], [], e3.channels, [0, 1, 2], []);
    if (Le.has(a3) && Le.has(n3)) return carryForwardMissingComponents(o3, [0, 1, 2], [], e3.channels, [0, 1, 2], []);
    switch (a3) {
      case xe.HSL:
        switch (n3) {
          case xe.HWB:
            return carryForwardMissingComponents(o3, [0], [1, 2], e3.channels, [0], [1, 2]);
          case xe.Lab:
          case xe.OKLab:
            return carryForwardMissingComponents(o3, [0], [1, 2], e3.channels, [2], [0, 1]);
          case xe.LCH:
          case xe.OKLCH:
            return carryForwardMissingComponents(o3, [0, 1, 2], [], e3.channels, [2, 1, 0], []);
          default:
            return carryForwardMissingComponents(o3, [], [], e3.channels, [], []);
        }
      case xe.HWB:
        switch (n3) {
          case xe.HSL:
            return carryForwardMissingComponents(o3, [0], [1, 2], e3.channels, [0], [1, 2]);
          case xe.LCH:
          case xe.OKLCH:
            return carryForwardMissingComponents(o3, [2], [0, 1], e3.channels, [0], [1, 2]);
          default:
            return carryForwardMissingComponents(o3, [], [], e3.channels, [], []);
        }
      case xe.Lab:
      case xe.OKLab:
        switch (n3) {
          case xe.HSL:
            return carryForwardMissingComponents(o3, [2], [0, 1], e3.channels, [0], [1, 2]);
          case xe.Lab:
          case xe.OKLab:
            return carryForwardMissingComponents(o3, [0, 1, 2], [], e3.channels, [0, 1, 2], []);
          case xe.LCH:
          case xe.OKLCH:
            return carryForwardMissingComponents(o3, [0], [1, 2], e3.channels, [0], [1, 2]);
          default:
            return carryForwardMissingComponents(o3, [], [], e3.channels, [], []);
        }
      case xe.LCH:
      case xe.OKLCH:
        switch (n3) {
          case xe.HSL:
            return carryForwardMissingComponents(o3, [0, 1, 2], [], e3.channels, [2, 1, 0], []);
          case xe.HWB:
            return carryForwardMissingComponents(o3, [0], [1, 2], e3.channels, [2], [0, 1]);
          case xe.Lab:
          case xe.OKLab:
            return carryForwardMissingComponents(o3, [0], [1, 2], e3.channels, [0], [1, 2]);
          case xe.LCH:
          case xe.OKLCH:
            return carryForwardMissingComponents(o3, [0, 1, 2], [], e3.channels, [0, 1, 2], []);
          default:
            return carryForwardMissingComponents(o3, [], [], e3.channels, [], []);
        }
      default:
        return carryForwardMissingComponents(o3, [], [], e3.channels, [], []);
    }
  }
  function convertPowerlessComponentsToMissingComponents(e3, n3) {
    const a3 = [...e3];
    switch (n3) {
      case xe.HSL:
        (Number.isNaN(a3[1]) ? 0 : a3[1]) <= 1e-3 && (a3[0] = Number.NaN, !Number.isNaN(a3[1]) && a3[1] > 0 && (a3[1] = 0));
        break;
      case xe.HWB:
        Math.max(0, Number.isNaN(a3[1]) ? 0 : a3[1]) + Math.max(0, Number.isNaN(a3[2]) ? 0 : a3[2]) >= 99.999 && (a3[0] = Number.NaN, Math.max(0, Number.isNaN(a3[1]) ? 0 : a3[1]) + Math.max(0, Number.isNaN(a3[2]) ? 0 : a3[2]) < 100 && (Number.isNaN(a3[1]) ? a3[2] = 100 : Number.isNaN(a3[2]) ? a3[1] = 100 : a3[2] = 100 - a3[1]));
        break;
      case xe.LCH:
        (Number.isNaN(a3[1]) ? 0 : a3[1]) <= 15e-4 && (a3[2] = Number.NaN, !Number.isNaN(a3[1]) && a3[1] > 0 && (a3[1] = 0));
        break;
      case xe.OKLCH:
        (Number.isNaN(a3[1]) ? 0 : a3[1]) <= 4e-6 && (a3[2] = Number.NaN, !Number.isNaN(a3[1]) && a3[1] > 0 && (a3[1] = 0));
    }
    return a3;
  }
  function carryForwardMissingComponents(e3, n3, a3, o3, r3, t3) {
    if (n3.length < 3 && e3.every(Number.isNaN)) return [Number.NaN, Number.NaN, Number.NaN];
    const l3 = [...o3];
    for (let a4 = 0; a4 < n3.length; a4++) Number.isNaN(e3[n3[a4]]) && (l3[r3[a4]] = Number.NaN);
    if (a3.length && a3.every((n4) => Number.isNaN(e3[n4]))) for (let e4 = 0; e4 < t3.length; e4++) l3[t3[e4]] = Number.NaN;
    return l3;
  }
  function normalizeRelativeColorDataChannels(e3) {
    const n3 = /* @__PURE__ */ new Map();
    switch (e3.colorNotation) {
      case xe.RGB:
      case xe.HEX:
        n3.set("r", dummyNumberToken(255 * e3.channels[0])), n3.set("g", dummyNumberToken(255 * e3.channels[1])), n3.set("b", dummyNumberToken(255 * e3.channels[2])), "number" == typeof e3.alpha && n3.set("alpha", dummyNumberToken(e3.alpha));
        break;
      case xe.HSL:
        n3.set("h", dummyNumberToken(e3.channels[0])), n3.set("s", dummyNumberToken(e3.channels[1])), n3.set("l", dummyNumberToken(e3.channels[2])), "number" == typeof e3.alpha && n3.set("alpha", dummyNumberToken(e3.alpha));
        break;
      case xe.HWB:
        n3.set("h", dummyNumberToken(e3.channels[0])), n3.set("w", dummyNumberToken(e3.channels[1])), n3.set("b", dummyNumberToken(e3.channels[2])), "number" == typeof e3.alpha && n3.set("alpha", dummyNumberToken(e3.alpha));
        break;
      case xe.Lab:
      case xe.OKLab:
        n3.set("l", dummyNumberToken(e3.channels[0])), n3.set("a", dummyNumberToken(e3.channels[1])), n3.set("b", dummyNumberToken(e3.channels[2])), "number" == typeof e3.alpha && n3.set("alpha", dummyNumberToken(e3.alpha));
        break;
      case xe.LCH:
      case xe.OKLCH:
        n3.set("l", dummyNumberToken(e3.channels[0])), n3.set("c", dummyNumberToken(e3.channels[1])), n3.set("h", dummyNumberToken(e3.channels[2])), "number" == typeof e3.alpha && n3.set("alpha", dummyNumberToken(e3.alpha));
        break;
      case xe.sRGB:
      case xe.A98_RGB:
      case xe.Display_P3:
      case xe.Linear_Display_P3:
      case xe.Rec2020:
      case xe.Linear_sRGB:
      case xe.ProPhoto_RGB:
        n3.set("r", dummyNumberToken(e3.channels[0])), n3.set("g", dummyNumberToken(e3.channels[1])), n3.set("b", dummyNumberToken(e3.channels[2])), "number" == typeof e3.alpha && n3.set("alpha", dummyNumberToken(e3.alpha));
        break;
      case xe.XYZ_D50:
      case xe.XYZ_D65:
        n3.set("x", dummyNumberToken(e3.channels[0])), n3.set("y", dummyNumberToken(e3.channels[1])), n3.set("z", dummyNumberToken(e3.channels[2])), "number" == typeof e3.alpha && n3.set("alpha", dummyNumberToken(e3.alpha));
    }
    return n3;
  }
  function noneToZeroInRelativeColorDataChannels(e3) {
    const n3 = new Map(e3);
    for (const [a3, o3] of e3) Number.isNaN(o3[4].value) && n3.set(a3, dummyNumberToken(0));
    return n3;
  }
  function normalizeAfterColorConversion(e3, n3 = -1) {
    return e3 = convertNaNToZero(e3), e3 = [0 === n3 ? e3[0] : normalize(e3[0], 1, -Infinity, Infinity), 1 === n3 ? e3[1] : normalize(e3[1], 1, -Infinity, Infinity), 2 === n3 ? e3[2] : normalize(e3[2], 1, -Infinity, Infinity)], Number.isNaN(e3[n3]) || Number.isFinite(e3[n3]) || (e3[n3] = 0), e3;
  }
  function dummyNumberToken(a3) {
    return Number.isNaN(a3) ? [u2.Number, "none", -1, -1, { value: Number.NaN, type: a2.Number }] : [u2.Number, a3.toString(), -1, -1, { value: a3, type: a2.Number }];
  }
  function reducePrecisionOrNaN(e3, n3 = 7) {
    if (Number.isNaN(e3)) return e3;
    const a3 = Math.pow(10, n3);
    return Math.round(e3 * a3) / a3;
  }
  var He = /[A-Z]/g;
  function toLowerCaseAZ2(e3) {
    return e3.replace(He, (e4) => String.fromCharCode(e4.charCodeAt(0) + 32));
  }
  function normalize_Color_ChannelValues(t3, l3, s3) {
    if (isTokenIdent(t3) && "none" === toLowerCaseAZ2(t3[4].value)) return s3.syntaxFlags.add(De.HasNoneKeywords), [u2.Number, "none", t3[2], t3[3], { value: Number.NaN, type: a2.Number }];
    if (isTokenPercentage(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasPercentageValues), a3 = 3 === l3 ? normalize(t3[4].value, 100, 0, 1) : normalize(t3[4].value, 100, -Infinity, Infinity), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    if (isTokenNumber(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasNumberValues), a3 = 3 === l3 ? normalize(t3[4].value, 1, 0, 1) : normalize(t3[4].value, 1, -Infinity, Infinity), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    return false;
  }
  var Pe = /* @__PURE__ */ new Set(["srgb", "srgb-linear", "display-p3", "display-p3-linear", "a98-rgb", "prophoto-rgb", "rec2020", "xyz", "xyz-d50", "xyz-d65"]);
  function color$1(e3, n3) {
    const o3 = [], s3 = [], c3 = [], i3 = [];
    let u3, h2, N2 = false, m2 = false;
    const p = { colorNotation: xe.sRGB, channels: [0, 0, 0], alpha: 1, syntaxFlags: /* @__PURE__ */ new Set([]) };
    let b2 = o3;
    for (let r3 = 0; r3 < e3.value.length; r3++) {
      let v2 = e3.value[r3];
      if (isWhitespaceNode(v2) || isCommentNode(v2)) for (; isWhitespaceNode(e3.value[r3 + 1]) || isCommentNode(e3.value[r3 + 1]); ) r3++;
      else if (b2 === o3 && o3.length && (b2 = s3), b2 === s3 && s3.length && (b2 = c3), isTokenNode(v2) && isTokenDelim(v2.value) && "/" === v2.value[4].value) {
        if (b2 === i3) return false;
        b2 = i3;
      } else {
        if (isFunctionNode(v2)) {
          if (b2 === i3 && "var" === toLowerCaseAZ2(v2.getName())) {
            p.syntaxFlags.add(De.HasVariableAlpha), b2.push(v2);
            continue;
          }
          if (!Q.has(toLowerCaseAZ2(v2.getName()))) return false;
          const [[e4]] = calcFromComponentValues([[v2]], { censorIntoStandardRepresentableValues: true, globals: h2, precision: -1, toCanonicalUnits: true, rawPercentages: true });
          if (!e4 || !isTokenNode(e4) || !isTokenNumeric(e4.value)) return false;
          Number.isNaN(e4.value[4].value) && (e4.value[4].value = 0), v2 = e4;
        }
        if (b2 === o3 && 0 === o3.length && isTokenNode(v2) && isTokenIdent(v2.value) && Pe.has(toLowerCaseAZ2(v2.value[4].value))) {
          if (N2) return false;
          N2 = toLowerCaseAZ2(v2.value[4].value), p.colorNotation = colorSpaceNameToColorNotation(N2), m2 && (m2 = colorDataToForRelativeColorSyntax(m2, p.colorNotation), u3 = normalizeRelativeColorDataChannels(m2), h2 = noneToZeroInRelativeColorDataChannels(u3));
        } else if (b2 === o3 && 0 === o3.length && isTokenNode(v2) && isTokenIdent(v2.value) && "from" === toLowerCaseAZ2(v2.value[4].value)) {
          if (m2) return false;
          if (N2) return false;
          for (; isWhitespaceNode(e3.value[r3 + 1]) || isCommentNode(e3.value[r3 + 1]); ) r3++;
          if (r3++, v2 = e3.value[r3], m2 = n3(v2), false === m2) return false;
          m2.syntaxFlags.has(De.Experimental) && p.syntaxFlags.add(De.Experimental), p.syntaxFlags.add(De.RelativeColorSyntax);
        } else {
          if (!isTokenNode(v2)) return false;
          if (isTokenIdent(v2.value) && u3 && u3.has(toLowerCaseAZ2(v2.value[4].value))) {
            b2.push(new TokenNode(u3.get(toLowerCaseAZ2(v2.value[4].value))));
            continue;
          }
          b2.push(v2);
        }
      }
    }
    if (!N2) return false;
    if (1 !== b2.length) return false;
    if (1 !== o3.length || 1 !== s3.length || 1 !== c3.length) return false;
    if (!isTokenNode(o3[0]) || !isTokenNode(s3[0]) || !isTokenNode(c3[0])) return false;
    if (u3 && !u3.has("alpha")) return false;
    const v = normalize_Color_ChannelValues(o3[0].value, 0, p);
    if (!v || !isTokenNumber(v)) return false;
    const f2 = normalize_Color_ChannelValues(s3[0].value, 1, p);
    if (!f2 || !isTokenNumber(f2)) return false;
    const g2 = normalize_Color_ChannelValues(c3[0].value, 2, p);
    if (!g2 || !isTokenNumber(g2)) return false;
    const d2 = [v, f2, g2];
    if (1 === i3.length) if (p.syntaxFlags.add(De.HasAlpha), isTokenNode(i3[0])) {
      const e4 = normalize_Color_ChannelValues(i3[0].value, 3, p);
      if (!e4 || !isTokenNumber(e4)) return false;
      d2.push(e4);
    } else p.alpha = i3[0];
    else if (u3 && u3.has("alpha")) {
      const e4 = normalize_Color_ChannelValues(u3.get("alpha"), 3, p);
      if (!e4 || !isTokenNumber(e4)) return false;
      d2.push(e4);
    }
    return p.channels = [d2[0][4].value, d2[1][4].value, d2[2][4].value], 4 === d2.length && (p.alpha = d2[3][4].value), p;
  }
  function colorSpaceNameToColorNotation(e3) {
    switch (e3) {
      case "srgb":
        return xe.sRGB;
      case "srgb-linear":
        return xe.Linear_sRGB;
      case "display-p3":
        return xe.Display_P3;
      case "display-p3-linear":
        return xe.Linear_Display_P3;
      case "a98-rgb":
        return xe.A98_RGB;
      case "prophoto-rgb":
        return xe.ProPhoto_RGB;
      case "rec2020":
        return xe.Rec2020;
      case "xyz":
      case "xyz-d65":
        return xe.XYZ_D65;
      case "xyz-d50":
        return xe.XYZ_D50;
      default:
        throw new Error("Unknown color space name: " + e3);
    }
  }
  var Ze = /* @__PURE__ */ new Set(["srgb", "srgb-linear", "display-p3", "display-p3-linear", "a98-rgb", "prophoto-rgb", "rec2020", "lab", "oklab", "xyz", "xyz-d50", "xyz-d65"]);
  var Me = /* @__PURE__ */ new Set(["hsl", "hwb", "lch", "oklch"]);
  var $e = /* @__PURE__ */ new Set(["shorter", "longer", "increasing", "decreasing"]);
  function colorMix(e3, n3) {
    let o3 = null, r3 = null, t3 = null, l3 = false;
    for (let c3 = 0; c3 < e3.value.length; c3++) {
      const i3 = e3.value[c3];
      if (!isWhiteSpaceOrCommentNode(i3)) {
        if (!(o3 || isTokenNode(i3) && isTokenIdent(i3.value) && "in" === toLowerCaseAZ2(i3.value[4].value))) return colorMixRectangular("oklab", colorMixComponents(e3.value, n3));
        if (isTokenNode(i3) && isTokenIdent(i3.value)) {
          if (!o3 && "in" === toLowerCaseAZ2(i3.value[4].value)) {
            o3 = i3;
            continue;
          }
          if (o3 && !r3) {
            r3 = toLowerCaseAZ2(i3.value[4].value);
            continue;
          }
          if (o3 && r3 && !t3 && Me.has(r3)) {
            t3 = toLowerCaseAZ2(i3.value[4].value);
            continue;
          }
          if (o3 && r3 && t3 && !l3 && "hue" === toLowerCaseAZ2(i3.value[4].value)) {
            l3 = true;
            continue;
          }
          return false;
        }
        return !(!isTokenNode(i3) || !isTokenComma(i3.value)) && (!!r3 && (t3 || l3 ? !!(t3 && l3 && Me.has(r3) && $e.has(t3)) && colorMixPolar(r3, t3, colorMixComponents(e3.value.slice(c3 + 1), n3)) : Ze.has(r3) ? colorMixRectangular(r3, colorMixComponents(e3.value.slice(c3 + 1), n3)) : !!Me.has(r3) && colorMixPolar(r3, "shorter", colorMixComponents(e3.value.slice(c3 + 1), n3))));
      }
    }
    return false;
  }
  function colorMixComponents(e3, n3) {
    const a3 = [];
    let r3 = false, t3 = false;
    for (let c3 = 0; c3 < e3.length; c3++) {
      let i3 = e3[c3];
      if (!isWhiteSpaceOrCommentNode(i3)) {
        if (!isTokenNode(i3) || !isTokenComma(i3.value)) {
          if (!r3) {
            const e4 = n3(i3);
            if (e4) {
              r3 = e4;
              continue;
            }
          }
          if (!t3) {
            if (isFunctionNode(i3) && Q.has(toLowerCaseAZ2(i3.getName()))) {
              if ([[i3]] = calcFromComponentValues([[i3]], { censorIntoStandardRepresentableValues: true, precision: -1, toCanonicalUnits: true, rawPercentages: true }), !i3 || !isTokenNode(i3) || !isTokenNumeric(i3.value)) return false;
              Number.isNaN(i3.value[4].value) && (i3.value[4].value = 0);
            }
            if (isTokenNode(i3) && isTokenPercentage(i3.value) && i3.value[4].value >= 0) {
              t3 = i3.value[4].value;
              continue;
            }
          }
          return false;
        }
        if (!r3) return false;
        a3.push({ color: r3, percentage: t3 }), r3 = false, t3 = false;
      }
    }
    return !!r3 && (a3.push({ color: r3, percentage: t3 }), a3);
  }
  function colorMixRectangular(e3, n3) {
    if (!n3 || !n3.length) return false;
    for (const e4 of n3) if (e4.percentage && (e4.percentage < 0 || e4.percentage > 100)) return false;
    const { items: a3, leftover: o3 } = normalizeMixPercentages2(n3, true), r3 = 1 - o3 / 100;
    let t3;
    switch (e3) {
      case "srgb":
        t3 = xe.RGB;
        break;
      case "srgb-linear":
        t3 = xe.Linear_sRGB;
        break;
      case "display-p3":
        t3 = xe.Display_P3;
        break;
      case "display-p3-linear":
        t3 = xe.Linear_Display_P3;
        break;
      case "a98-rgb":
        t3 = xe.A98_RGB;
        break;
      case "prophoto-rgb":
        t3 = xe.ProPhoto_RGB;
        break;
      case "rec2020":
        t3 = xe.Rec2020;
        break;
      case "lab":
        t3 = xe.Lab;
        break;
      case "oklab":
        t3 = xe.OKLab;
        break;
      case "xyz-d50":
        t3 = xe.XYZ_D50;
        break;
      case "xyz":
      case "xyz-d65":
        t3 = xe.XYZ_D65;
        break;
      default:
        return false;
    }
    if (1 === a3.length) {
      const e4 = colorDataToForInterpolation(a3[0].color, t3);
      return e4.colorNotation = t3, e4.syntaxFlags.add(De.ColorMixVariadic), e4.syntaxFlags.add(De.ColorMix), "number" != typeof e4.alpha ? false : (e4.alpha = e4.alpha * r3, e4);
    }
    for (a3.reverse(); a3.length >= 2; ) {
      const e4 = a3.pop(), n4 = a3.pop();
      if (!e4 || !n4) return false;
      const o4 = e4.percentage + n4.percentage, r4 = o4 > 0 ? n4.percentage / o4 : 0.5, l4 = colorMixRectangularPair(t3, e4.color, n4.color, r4);
      if (!l4) return false;
      a3.push({ color: l4, percentage: o4 });
    }
    const l3 = a3[0]?.color;
    return !!l3 && ("number" == typeof l3.alpha && (l3.alpha = l3.alpha * r3, n3.some((e4) => e4.color.syntaxFlags.has(De.Experimental)) && l3.syntaxFlags.add(De.Experimental), 2 !== n3.length && l3.syntaxFlags.add(De.ColorMixVariadic), l3));
  }
  function colorMixRectangularPair(e3, n3, a3, o3) {
    let r3 = n3.alpha;
    if ("number" != typeof r3) return false;
    let t3 = a3.alpha;
    if ("number" != typeof t3) return false;
    r3 = Number.isNaN(r3) ? t3 : r3, t3 = Number.isNaN(t3) ? r3 : t3;
    const l3 = colorDataToForInterpolation(n3, e3).channels, s3 = colorDataToForInterpolation(a3, e3).channels;
    l3[0] = fillInMissingComponent(l3[0], s3[0]), s3[0] = fillInMissingComponent(s3[0], l3[0]), l3[1] = fillInMissingComponent(l3[1], s3[1]), s3[1] = fillInMissingComponent(s3[1], l3[1]), l3[2] = fillInMissingComponent(l3[2], s3[2]), s3[2] = fillInMissingComponent(s3[2], l3[2]), l3[0] = premultiply(l3[0], r3), l3[1] = premultiply(l3[1], r3), l3[2] = premultiply(l3[2], r3), s3[0] = premultiply(s3[0], t3), s3[1] = premultiply(s3[1], t3), s3[2] = premultiply(s3[2], t3);
    const c3 = interpolate(r3, t3, o3);
    return { colorNotation: e3, channels: [un_premultiply(interpolate(l3[0], s3[0], o3), c3), un_premultiply(interpolate(l3[1], s3[1], o3), c3), un_premultiply(interpolate(l3[2], s3[2], o3), c3)], alpha: c3, syntaxFlags: /* @__PURE__ */ new Set([De.ColorMix]) };
  }
  function colorMixPolar(e3, n3, a3) {
    if (!a3 || !a3.length) return false;
    for (const e4 of a3) if (e4.percentage && (e4.percentage < 0 || e4.percentage > 100)) return false;
    const { items: o3, leftover: r3 } = normalizeMixPercentages2(a3, true), t3 = 1 - r3 / 100;
    let l3;
    switch (e3) {
      case "hsl":
        l3 = xe.HSL;
        break;
      case "hwb":
        l3 = xe.HWB;
        break;
      case "lch":
        l3 = xe.LCH;
        break;
      case "oklch":
        l3 = xe.OKLCH;
        break;
      default:
        return false;
    }
    if (1 === o3.length) {
      const e4 = colorDataToForInterpolation(o3[0].color, l3);
      return e4.colorNotation = l3, e4.syntaxFlags.add(De.ColorMixVariadic), e4.syntaxFlags.add(De.ColorMix), "number" != typeof e4.alpha ? false : (e4.alpha = e4.alpha * t3, e4);
    }
    for (o3.reverse(); o3.length >= 2; ) {
      const e4 = o3.pop(), a4 = o3.pop();
      if (!e4 || !a4) return false;
      const r4 = e4.percentage + a4.percentage, t4 = r4 > 0 ? a4.percentage / r4 : 0.5, s4 = colorMixPolarPair(l3, n3, e4.color, a4.color, t4);
      if (!s4) return false;
      o3.push({ color: s4, percentage: r4 });
    }
    const s3 = o3[0]?.color;
    return !!s3 && ("number" == typeof s3.alpha && (s3.alpha = s3.alpha * t3, a3.some((e4) => e4.color.syntaxFlags.has(De.Experimental)) && s3.syntaxFlags.add(De.Experimental), 2 !== a3.length && s3.syntaxFlags.add(De.ColorMixVariadic), s3));
  }
  function colorMixPolarPair(e3, n3, a3, o3, r3) {
    let t3 = 0, l3 = 0, s3 = 0, c3 = 0, i3 = 0, u3 = 0, h2 = a3.alpha;
    if ("number" != typeof h2) return false;
    let N2 = o3.alpha;
    if ("number" != typeof N2) return false;
    h2 = Number.isNaN(h2) ? N2 : h2, N2 = Number.isNaN(N2) ? h2 : N2;
    const m2 = colorDataToForInterpolation(a3, e3).channels, p = colorDataToForInterpolation(o3, e3).channels;
    switch (e3) {
      case xe.HSL:
      case xe.HWB:
        t3 = m2[0], l3 = p[0], s3 = m2[1], c3 = p[1], i3 = m2[2], u3 = p[2];
        break;
      case xe.LCH:
      case xe.OKLCH:
        s3 = m2[0], c3 = p[0], i3 = m2[1], u3 = p[1], t3 = m2[2], l3 = p[2];
    }
    if (s3 = fillInMissingComponent(s3, c3), c3 = fillInMissingComponent(c3, s3), i3 = fillInMissingComponent(i3, u3), u3 = fillInMissingComponent(u3, i3), t3 = fillInMissingComponent(t3, l3), l3 = fillInMissingComponent(l3, t3), Number.isNaN(t3) && Number.isNaN(l3)) ;
    else {
      const e4 = l3 - t3;
      switch (n3) {
        case "shorter":
          e4 > 180 ? t3 += 360 : e4 < -180 && (l3 += 360);
          break;
        case "longer":
          -180 < e4 && e4 < 180 && (e4 > 0 ? t3 += 360 : l3 += 360);
          break;
        case "increasing":
          e4 < 0 && (l3 += 360);
          break;
        case "decreasing":
          e4 > 0 && (t3 += 360);
          break;
        default:
          throw new Error("Unknown hue interpolation method");
      }
    }
    s3 = premultiply(s3, h2), i3 = premultiply(i3, h2), c3 = premultiply(c3, N2), u3 = premultiply(u3, N2);
    let b2 = [0, 0, 0];
    const v = interpolate(h2, N2, r3);
    switch (e3) {
      case xe.HSL:
      case xe.HWB:
        b2 = [interpolate(t3, l3, r3), un_premultiply(interpolate(s3, c3, r3), v), un_premultiply(interpolate(i3, u3, r3), v)];
        break;
      case xe.LCH:
      case xe.OKLCH:
        b2 = [un_premultiply(interpolate(s3, c3, r3), v), un_premultiply(interpolate(i3, u3, r3), v), interpolate(t3, l3, r3)];
    }
    return { colorNotation: e3, channels: b2, alpha: v, syntaxFlags: /* @__PURE__ */ new Set([De.ColorMix]) };
  }
  function fillInMissingComponent(e3, n3) {
    return Number.isNaN(e3) ? n3 : e3;
  }
  function interpolate(e3, n3, a3) {
    return e3 * (1 - a3) + n3 * a3;
  }
  function premultiply(e3, n3) {
    return Number.isNaN(n3) ? e3 : Number.isNaN(e3) ? Number.NaN : e3 * n3;
  }
  function un_premultiply(e3, n3) {
    return 0 === n3 || Number.isNaN(n3) ? e3 : Number.isNaN(e3) ? Number.NaN : e3 / n3;
  }
  function normalizeMixPercentages2(e3, n3 = false) {
    let a3 = 0, o3 = 0;
    for (const n4 of e3) n4.percentage && (a3 += n4.percentage), false === n4.percentage && o3++;
    a3 = Math.min(100, a3);
    for (const n4 of e3) false === n4.percentage && (n4.percentage = (100 - a3) / o3);
    const r3 = e3.slice();
    let t3 = 0;
    for (const e4 of r3) t3 += e4.percentage;
    if (t3 > 100 || t3 > 0 && n3) for (const e4 of r3) e4.percentage = e4.percentage * (100 / t3);
    let l3 = 0;
    return t3 < 100 && (l3 = 100 - t3), { items: r3, leftover: l3 };
  }
  function hex(e3) {
    const n3 = toLowerCaseAZ2(e3[4].value);
    if (n3.match(/[^a-f0-9]/)) return false;
    const a3 = { colorNotation: xe.HEX, channels: [0, 0, 0], alpha: 1, syntaxFlags: /* @__PURE__ */ new Set([De.Hex]) }, o3 = n3.length;
    if (3 === o3) {
      const e4 = n3[0], o4 = n3[1], r3 = n3[2];
      return a3.channels = [parseInt(e4 + e4, 16) / 255, parseInt(o4 + o4, 16) / 255, parseInt(r3 + r3, 16) / 255], a3;
    }
    if (6 === o3) {
      const e4 = n3[0] + n3[1], o4 = n3[2] + n3[3], r3 = n3[4] + n3[5];
      return a3.channels = [parseInt(e4, 16) / 255, parseInt(o4, 16) / 255, parseInt(r3, 16) / 255], a3;
    }
    if (4 === o3) {
      const e4 = n3[0], o4 = n3[1], r3 = n3[2], t3 = n3[3];
      return a3.channels = [parseInt(e4 + e4, 16) / 255, parseInt(o4 + o4, 16) / 255, parseInt(r3 + r3, 16) / 255], a3.alpha = parseInt(t3 + t3, 16) / 255, a3.syntaxFlags.add(De.HasAlpha), a3;
    }
    if (8 === o3) {
      const e4 = n3[0] + n3[1], o4 = n3[2] + n3[3], r3 = n3[4] + n3[5], t3 = n3[6] + n3[7];
      return a3.channels = [parseInt(e4, 16) / 255, parseInt(o4, 16) / 255, parseInt(r3, 16) / 255], a3.alpha = parseInt(t3, 16) / 255, a3.syntaxFlags.add(De.HasAlpha), a3;
    }
    return false;
  }
  function normalizeHue$1(a3) {
    if (isTokenNumber(a3)) return Number.isNaN(a3[4].value) || Number.isFinite(a3[4].value) || (a3[4].value = 0), a3[4].value = a3[4].value % 360, a3[4].value < 0 && (a3[4].value += 360), a3[1] = a3[4].value.toString(), a3;
    if (isTokenDimension(a3)) {
      let o3 = a3[4].value;
      switch (toLowerCaseAZ2(a3[4].unit)) {
        case "deg":
          break;
        case "rad":
          o3 = 180 * a3[4].value / Math.PI;
          break;
        case "grad":
          o3 = 0.9 * a3[4].value;
          break;
        case "turn":
          o3 = 360 * a3[4].value;
          break;
        default:
          return false;
      }
      return Number.isNaN(a3[4].value) || Number.isFinite(a3[4].value) || (a3[4].value = 0), o3 %= 360, o3 < 0 && (o3 += 360), [u2.Number, o3.toString(), a3[2], a3[3], { value: o3, type: a2.Number }];
    }
    return false;
  }
  function normalize_legacy_HSL_ChannelValues(a3, t3, l3) {
    if (0 === t3) {
      const e3 = normalizeHue$1(a3);
      return false !== e3 && (isTokenDimension(a3) && l3.syntaxFlags.add(De.HasDimensionValues), e3);
    }
    if (isTokenPercentage(a3)) {
      let o3;
      return 3 === t3 ? l3.syntaxFlags.add(De.HasPercentageAlpha) : l3.syntaxFlags.add(De.HasPercentageValues), o3 = 3 === t3 ? normalize(a3[4].value, 100, 0, 1) : normalize(a3[4].value, 1, 0, 100), [u2.Number, o3.toString(), a3[2], a3[3], { value: o3, type: a2.Number }];
    }
    if (isTokenNumber(a3)) {
      if (3 !== t3) return false;
      let o3;
      return o3 = normalize(a3[4].value, 1, 0, 3 === t3 ? 1 : 100), [u2.Number, o3.toString(), a3[2], a3[3], { value: o3, type: a2.Number }];
    }
    return false;
  }
  function normalize_modern_HSL_ChannelValues(t3, l3, s3) {
    if (isTokenIdent(t3) && "none" === toLowerCaseAZ2(t3[4].value)) return s3.syntaxFlags.add(De.HasNoneKeywords), [u2.Number, "none", t3[2], t3[3], { value: Number.NaN, type: a2.Number }];
    if (0 === l3) {
      const e3 = normalizeHue$1(t3);
      return false !== e3 && (isTokenDimension(t3) && s3.syntaxFlags.add(De.HasDimensionValues), e3);
    }
    if (isTokenPercentage(t3)) {
      let a3;
      return 3 === l3 ? s3.syntaxFlags.add(De.HasPercentageAlpha) : s3.syntaxFlags.add(De.HasPercentageValues), a3 = 3 === l3 ? normalize(t3[4].value, 100, 0, 1) : normalize(t3[4].value, 1, 1 === l3 ? 0 : -Infinity, Infinity), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    if (isTokenNumber(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasNumberValues), a3 = 3 === l3 ? normalize(t3[4].value, 1, 0, 1) : normalize(t3[4].value, 1, 1 === l3 ? 0 : -Infinity, Infinity), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    return false;
  }
  function threeChannelLegacySyntax(e3, n3, a3, o3) {
    const t3 = [], c3 = [], i3 = [], u3 = [], h2 = { colorNotation: a3, channels: [0, 0, 0], alpha: 1, syntaxFlags: new Set(o3) };
    let N2 = t3;
    for (let n4 = 0; n4 < e3.value.length; n4++) {
      let a4 = e3.value[n4];
      if (!isWhitespaceNode(a4) && !isCommentNode(a4)) {
        if (isTokenNode(a4) && isTokenComma(a4.value)) {
          if (N2 === t3) {
            N2 = c3;
            continue;
          }
          if (N2 === c3) {
            N2 = i3;
            continue;
          }
          if (N2 === i3) {
            N2 = u3;
            continue;
          }
          if (N2 === u3) return false;
        }
        if (isFunctionNode(a4)) {
          if (N2 === u3 && "var" === a4.getName().toLowerCase()) {
            h2.syntaxFlags.add(De.HasVariableAlpha), N2.push(a4);
            continue;
          }
          if (!Q.has(a4.getName().toLowerCase())) return false;
          const [[e4]] = calcFromComponentValues([[a4]], { censorIntoStandardRepresentableValues: true, precision: -1, toCanonicalUnits: true, rawPercentages: true });
          if (!e4 || !isTokenNode(e4) || !isTokenNumeric(e4.value)) return false;
          Number.isNaN(e4.value[4].value) && (e4.value[4].value = 0), a4 = e4;
        }
        if (!isTokenNode(a4)) return false;
        N2.push(a4);
      }
    }
    if (1 !== N2.length) return false;
    if (1 !== t3.length || 1 !== c3.length || 1 !== i3.length) return false;
    if (!isTokenNode(t3[0]) || !isTokenNode(c3[0]) || !isTokenNode(i3[0])) return false;
    const m2 = n3(t3[0].value, 0, h2);
    if (!m2 || !isTokenNumber(m2)) return false;
    const p = n3(c3[0].value, 1, h2);
    if (!p || !isTokenNumber(p)) return false;
    const b2 = n3(i3[0].value, 2, h2);
    if (!b2 || !isTokenNumber(b2)) return false;
    const v = [m2, p, b2];
    if (1 === u3.length) if (h2.syntaxFlags.add(De.HasAlpha), isTokenNode(u3[0])) {
      const e4 = n3(u3[0].value, 3, h2);
      if (!e4 || !isTokenNumber(e4)) return false;
      v.push(e4);
    } else h2.alpha = u3[0];
    return h2.channels = [v[0][4].value, v[1][4].value, v[2][4].value], 4 === v.length && (h2.alpha = v[3][4].value), h2;
  }
  function threeChannelSpaceSeparated(e3, n3, o3, s3, c3) {
    const i3 = [], u3 = [], h2 = [], N2 = [];
    let m2, p, b2 = false;
    const v = { colorNotation: o3, channels: [0, 0, 0], alpha: 1, syntaxFlags: new Set(s3) };
    let f2 = i3;
    for (let n4 = 0; n4 < e3.value.length; n4++) {
      let r3 = e3.value[n4];
      if (isWhitespaceNode(r3) || isCommentNode(r3)) for (; isWhitespaceNode(e3.value[n4 + 1]) || isCommentNode(e3.value[n4 + 1]); ) n4++;
      else if (f2 === i3 && i3.length && (f2 = u3), f2 === u3 && u3.length && (f2 = h2), isTokenNode(r3) && isTokenDelim(r3.value) && "/" === r3.value[4].value) {
        if (f2 === N2) return false;
        f2 = N2;
      } else {
        if (isFunctionNode(r3)) {
          if (f2 === N2 && "var" === r3.getName().toLowerCase()) {
            v.syntaxFlags.add(De.HasVariableAlpha), f2.push(r3);
            continue;
          }
          if (!Q.has(r3.getName().toLowerCase())) return false;
          const [[e4]] = calcFromComponentValues([[r3]], { censorIntoStandardRepresentableValues: true, globals: p, precision: -1, toCanonicalUnits: true, rawPercentages: true });
          if (!e4 || !isTokenNode(e4) || !isTokenNumeric(e4.value)) return false;
          Number.isNaN(e4.value[4].value) && (e4.value[4].value = 0), r3 = e4;
        }
        if (f2 === i3 && 0 === i3.length && isTokenNode(r3) && isTokenIdent(r3.value) && "from" === r3.value[4].value.toLowerCase()) {
          if (b2) return false;
          for (; isWhitespaceNode(e3.value[n4 + 1]) || isCommentNode(e3.value[n4 + 1]); ) n4++;
          if (n4++, r3 = e3.value[n4], b2 = c3(r3), false === b2) return false;
          b2.syntaxFlags.has(De.Experimental) && v.syntaxFlags.add(De.Experimental), v.syntaxFlags.add(De.RelativeColorSyntax), b2 = colorDataToForRelativeColorSyntax(b2, o3), m2 = normalizeRelativeColorDataChannels(b2), p = noneToZeroInRelativeColorDataChannels(m2);
        } else {
          if (!isTokenNode(r3)) return false;
          if (isTokenIdent(r3.value) && m2) {
            const e4 = r3.value[4].value.toLowerCase();
            if (m2.has(e4)) {
              f2.push(new TokenNode(m2.get(e4)));
              continue;
            }
          }
          f2.push(r3);
        }
      }
    }
    if (1 !== f2.length) return false;
    if (1 !== i3.length || 1 !== u3.length || 1 !== h2.length) return false;
    if (!isTokenNode(i3[0]) || !isTokenNode(u3[0]) || !isTokenNode(h2[0])) return false;
    if (m2 && !m2.has("alpha")) return false;
    const g2 = n3(i3[0].value, 0, v);
    if (!g2 || !isTokenNumber(g2)) return false;
    const d2 = n3(u3[0].value, 1, v);
    if (!d2 || !isTokenNumber(d2)) return false;
    const y2 = n3(h2[0].value, 2, v);
    if (!y2 || !isTokenNumber(y2)) return false;
    const _3 = [g2, d2, y2];
    if (1 === N2.length) if (v.syntaxFlags.add(De.HasAlpha), isTokenNode(N2[0])) {
      const e4 = n3(N2[0].value, 3, v);
      if (!e4 || !isTokenNumber(e4)) return false;
      _3.push(e4);
    } else v.alpha = N2[0];
    else if (m2 && m2.has("alpha")) {
      const e4 = n3(m2.get("alpha"), 3, v);
      if (!e4 || !isTokenNumber(e4)) return false;
      _3.push(e4);
    }
    return v.channels = [_3[0][4].value, _3[1][4].value, _3[2][4].value], 4 === _3.length && (v.alpha = _3[3][4].value), v;
  }
  function hsl(e3, n3) {
    if (e3.value.some((e4) => isTokenNode(e4) && isTokenComma(e4.value))) {
      const n4 = hslCommaSeparated(e3);
      if (false !== n4) return n4;
    }
    {
      const a3 = hslSpaceSeparated(e3, n3);
      if (false !== a3) return a3;
    }
    return false;
  }
  function hslCommaSeparated(e3) {
    return threeChannelLegacySyntax(e3, normalize_legacy_HSL_ChannelValues, xe.HSL, [De.LegacyHSL]);
  }
  function hslSpaceSeparated(e3, n3) {
    return threeChannelSpaceSeparated(e3, normalize_modern_HSL_ChannelValues, xe.HSL, [], n3);
  }
  function normalize_HWB_ChannelValues(t3, l3, s3) {
    if (isTokenIdent(t3) && "none" === toLowerCaseAZ2(t3[4].value)) return s3.syntaxFlags.add(De.HasNoneKeywords), [u2.Number, "none", t3[2], t3[3], { value: Number.NaN, type: a2.Number }];
    if (0 === l3) {
      const e3 = normalizeHue$1(t3);
      return false !== e3 && (isTokenDimension(t3) && s3.syntaxFlags.add(De.HasDimensionValues), e3);
    }
    if (isTokenPercentage(t3)) {
      let a3;
      return 3 === l3 ? s3.syntaxFlags.add(De.HasPercentageAlpha) : s3.syntaxFlags.add(De.HasPercentageValues), a3 = 3 === l3 ? normalize(t3[4].value, 100, 0, 1) : normalize(t3[4].value, 1, -Infinity, Infinity), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    if (isTokenNumber(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasNumberValues), a3 = 3 === l3 ? normalize(t3[4].value, 1, 0, 1) : normalize(t3[4].value, 1, -Infinity, Infinity), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    return false;
  }
  function normalize_Lab_ChannelValues(t3, l3, s3) {
    if (isTokenIdent(t3) && "none" === toLowerCaseAZ2(t3[4].value)) return s3.syntaxFlags.add(De.HasNoneKeywords), [u2.Number, "none", t3[2], t3[3], { value: Number.NaN, type: a2.Number }];
    if (isTokenPercentage(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasPercentageValues), a3 = 1 === l3 || 2 === l3 ? normalize(t3[4].value, 0.8, -Infinity, Infinity) : 3 === l3 ? normalize(t3[4].value, 100, 0, 1) : normalize(t3[4].value, 1, 0, 100), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    if (isTokenNumber(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasNumberValues), a3 = 1 === l3 || 2 === l3 ? normalize(t3[4].value, 1, -Infinity, Infinity) : normalize(t3[4].value, 1, 0, 3 === l3 ? 1 : 100), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    return false;
  }
  function lab(e3, n3) {
    return threeChannelSpaceSeparated(e3, normalize_Lab_ChannelValues, xe.Lab, [], n3);
  }
  function normalize_LCH_ChannelValues(t3, l3, s3) {
    if (isTokenIdent(t3) && "none" === toLowerCaseAZ2(t3[4].value)) return s3.syntaxFlags.add(De.HasNoneKeywords), [u2.Number, "none", t3[2], t3[3], { value: Number.NaN, type: a2.Number }];
    if (2 === l3) {
      const e3 = normalizeHue$1(t3);
      return false !== e3 && (isTokenDimension(t3) && s3.syntaxFlags.add(De.HasDimensionValues), e3);
    }
    if (isTokenPercentage(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasPercentageValues), a3 = 1 === l3 ? normalize(t3[4].value, 100 / 150, 0, Infinity) : 3 === l3 ? normalize(t3[4].value, 100, 0, 1) : normalize(t3[4].value, 1, 0, 100), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    if (isTokenNumber(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasNumberValues), a3 = normalize(t3[4].value, 1, 0, 1 === l3 ? Infinity : 3 === l3 ? 1 : 100), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    return false;
  }
  function lch(e3, n3) {
    return threeChannelSpaceSeparated(e3, normalize_LCH_ChannelValues, xe.LCH, [], n3);
  }
  var Fe = /* @__PURE__ */ new Map();
  for (const [e3, n3] of Object.entries(G)) Fe.set(e3, n3);
  function namedColor(e3) {
    const n3 = Fe.get(toLowerCaseAZ2(e3));
    return !!n3 && { colorNotation: xe.RGB, channels: [n3[0] / 255, n3[1] / 255, n3[2] / 255], alpha: 1, syntaxFlags: /* @__PURE__ */ new Set([De.ColorKeyword, De.NamedColor]) };
  }
  function normalize_OKLab_ChannelValues(t3, l3, s3) {
    if (isTokenIdent(t3) && "none" === toLowerCaseAZ2(t3[4].value)) return s3.syntaxFlags.add(De.HasNoneKeywords), [u2.Number, "none", t3[2], t3[3], { value: Number.NaN, type: a2.Number }];
    if (isTokenPercentage(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasPercentageValues), a3 = 1 === l3 || 2 === l3 ? normalize(t3[4].value, 250, -Infinity, Infinity) : normalize(t3[4].value, 100, 0, 1), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    if (isTokenNumber(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasNumberValues), a3 = 1 === l3 || 2 === l3 ? normalize(t3[4].value, 1, -Infinity, Infinity) : normalize(t3[4].value, 1, 0, 1), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    return false;
  }
  function oklab(e3, n3) {
    return threeChannelSpaceSeparated(e3, normalize_OKLab_ChannelValues, xe.OKLab, [], n3);
  }
  function normalize_OKLCH_ChannelValues(t3, l3, s3) {
    if (isTokenIdent(t3) && "none" === toLowerCaseAZ2(t3[4].value)) return s3.syntaxFlags.add(De.HasNoneKeywords), [u2.Number, "none", t3[2], t3[3], { value: Number.NaN, type: a2.Number }];
    if (2 === l3) {
      const e3 = normalizeHue$1(t3);
      return false !== e3 && (isTokenDimension(t3) && s3.syntaxFlags.add(De.HasDimensionValues), e3);
    }
    if (isTokenPercentage(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasPercentageValues), a3 = 1 === l3 ? normalize(t3[4].value, 250, 0, Infinity) : normalize(t3[4].value, 100, 0, 1), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    if (isTokenNumber(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasNumberValues), a3 = normalize(t3[4].value, 1, 0, 1 === l3 ? Infinity : 1), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    return false;
  }
  function oklch(e3, n3) {
    return threeChannelSpaceSeparated(e3, normalize_OKLCH_ChannelValues, xe.OKLCH, [], n3);
  }
  function normalize_legacy_sRGB_ChannelValues(a3, t3, l3) {
    if (isTokenPercentage(a3)) {
      3 === t3 ? l3.syntaxFlags.add(De.HasPercentageAlpha) : l3.syntaxFlags.add(De.HasPercentageValues);
      const o3 = normalize(a3[4].value, 100, 0, 1);
      return [u2.Number, o3.toString(), a3[2], a3[3], { value: o3, type: a2.Number }];
    }
    if (isTokenNumber(a3)) {
      let o3;
      return 3 !== t3 && l3.syntaxFlags.add(De.HasNumberValues), o3 = normalize(a3[4].value, 3 === t3 ? 1 : 255, 0, 1), [u2.Number, o3.toString(), a3[2], a3[3], { value: o3, type: a2.Number }];
    }
    return false;
  }
  function normalize_modern_sRGB_ChannelValues(t3, l3, s3) {
    if (isTokenIdent(t3) && "none" === t3[4].value.toLowerCase()) return s3.syntaxFlags.add(De.HasNoneKeywords), [u2.Number, "none", t3[2], t3[3], { value: Number.NaN, type: a2.Number }];
    if (isTokenPercentage(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasPercentageValues), a3 = 3 === l3 ? normalize(t3[4].value, 100, 0, 1) : normalize(t3[4].value, 100, -Infinity, Infinity), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    if (isTokenNumber(t3)) {
      let a3;
      return 3 !== l3 && s3.syntaxFlags.add(De.HasNumberValues), a3 = 3 === l3 ? normalize(t3[4].value, 1, 0, 1) : normalize(t3[4].value, 255, -Infinity, Infinity), [u2.Number, a3.toString(), t3[2], t3[3], { value: a3, type: a2.Number }];
    }
    return false;
  }
  function rgb(e3, n3) {
    if (e3.value.some((e4) => isTokenNode(e4) && isTokenComma(e4.value))) {
      const n4 = rgbCommaSeparated(e3);
      if (false !== n4) return (!n4.syntaxFlags.has(De.HasNumberValues) || !n4.syntaxFlags.has(De.HasPercentageValues)) && n4;
    } else {
      const a3 = rgbSpaceSeparated(e3, n3);
      if (false !== a3) return a3;
    }
    return false;
  }
  function rgbCommaSeparated(e3) {
    return threeChannelLegacySyntax(e3, normalize_legacy_sRGB_ChannelValues, xe.RGB, [De.LegacyRGB]);
  }
  function rgbSpaceSeparated(e3, n3) {
    return threeChannelSpaceSeparated(e3, normalize_modern_sRGB_ChannelValues, xe.RGB, [], n3);
  }
  function XYZ_D65_to_sRGB_Gamut(e3) {
    const n3 = XYZ_D65_to_sRGB(e3);
    if (inGamut(n3)) return clip(n3);
    let a3 = e3;
    return a3 = XYZ_D65_to_OKLCH(a3), a3[0] < 1e-6 && (a3 = [0, 0, 0]), a3[0] > 0.999999 && (a3 = [1, 0, 0]), gam_sRGB(mapGamutRayTrace(a3, oklch_to_lin_srgb, lin_srgb_to_oklch));
  }
  function oklch_to_lin_srgb(e3) {
    return e3 = OKLCH_to_OKLab(e3), e3 = OKLab_to_XYZ(e3), XYZ_to_lin_sRGB(e3);
  }
  function lin_srgb_to_oklch(e3) {
    return e3 = lin_sRGB_to_XYZ(e3), e3 = XYZ_to_OKLab(e3), OKLab_to_OKLCH(e3);
  }
  function contrastColor(e3, n3) {
    let a3 = false;
    for (let o4 = 0; o4 < e3.value.length; o4++) {
      const r4 = e3.value[o4];
      if (!isWhitespaceNode(r4) && !isCommentNode(r4) && (a3 || (a3 = n3(r4), !a3))) return false;
    }
    if (!a3) return false;
    a3.channels = convertNaNToZero(a3.channels), a3.channels = XYZ_D65_to_sRGB_Gamut(colorData_to_XYZ_D65(a3).channels), a3.colorNotation = xe.sRGB;
    const o3 = { colorNotation: xe.RGB, channels: [0, 0, 0], alpha: 1, syntaxFlags: /* @__PURE__ */ new Set([De.ContrastColor, De.Experimental]) }, r3 = contrast_ratio_wcag_2_1(a3.channels, [1, 1, 1]), t3 = contrast_ratio_wcag_2_1(a3.channels, [0, 0, 0]);
    return o3.channels = r3 > t3 ? [1, 1, 1] : [0, 0, 0], o3;
  }
  function alpha(e3, n3) {
    let o3, s3, c3 = false, i3 = false, u3 = false;
    const h2 = { colorNotation: xe.sRGB, channels: [0, 0, 0], alpha: 1, syntaxFlags: /* @__PURE__ */ new Set([]) };
    for (let N2 = 0; N2 < e3.value.length; N2++) {
      let m2 = e3.value[N2];
      if (isWhitespaceNode(m2) || isCommentNode(m2)) for (; isWhitespaceNode(e3.value[N2 + 1]) || isCommentNode(e3.value[N2 + 1]); ) N2++;
      else if (u3 && !c3 && !i3 && isTokenNode(m2) && isTokenDelim(m2.value) && "/" === m2.value[4].value) c3 = true;
      else {
        if (isFunctionNode(m2) && Q.has(toLowerCaseAZ2(m2.getName()))) {
          const [[e4]] = calcFromComponentValues([[m2]], { censorIntoStandardRepresentableValues: true, globals: s3, precision: -1, toCanonicalUnits: true, rawPercentages: true });
          if (!e4 || !isTokenNode(e4) || !isTokenNumeric(e4.value)) return false;
          Number.isNaN(e4.value[4].value) && (e4.value[4].value = 0), m2 = e4;
        }
        if (c3 || i3 || !isTokenNode(m2) || !isTokenIdent(m2.value) || "from" !== toLowerCaseAZ2(m2.value[4].value)) {
          if (!c3) return false;
          if (i3) return false;
          if (isTokenNode(m2)) {
            if (isTokenIdent(m2.value) && "alpha" === toLowerCaseAZ2(m2.value[4].value) && o3 && o3.has("alpha")) {
              h2.alpha = o3.get("alpha")[4].value, i3 = true;
              continue;
            }
            const e4 = normalize_Color_ChannelValues(m2.value, 3, h2);
            if (!e4 || !isTokenNumber(e4)) return false;
            h2.alpha = e4[4].value, i3 = true;
            continue;
          }
          if (isFunctionNode(m2)) {
            const e4 = replaceComponentValues([[m2]], (e5) => {
              if (isTokenNode(e5) && isTokenIdent(e5.value) && "alpha" === toLowerCaseAZ2(e5.value[4].value) && o3 && o3.has("alpha")) return new TokenNode(o3.get("alpha"));
            });
            h2.alpha = e4[0][0], i3 = true;
            continue;
          }
          return false;
        }
        if (u3) return false;
        for (; isWhitespaceNode(e3.value[N2 + 1]) || isCommentNode(e3.value[N2 + 1]); ) N2++;
        if (N2++, m2 = e3.value[N2], u3 = n3(m2), false === u3) return false;
        o3 = normalizeRelativeColorDataChannels(u3), s3 = noneToZeroInRelativeColorDataChannels(o3), h2.syntaxFlags = new Set(u3.syntaxFlags), h2.syntaxFlags.add(De.RelativeAlphaSyntax), h2.channels = [...u3.channels], h2.colorNotation = u3.colorNotation, h2.alpha = u3.alpha;
      }
    }
    return !!o3 && (!!i3 && h2);
  }
  function color(e3) {
    if (isFunctionNode(e3)) {
      switch (toLowerCaseAZ2(e3.getName())) {
        case "rgb":
        case "rgba":
          return rgb(e3, color);
        case "hsl":
        case "hsla":
          return hsl(e3, color);
        case "hwb":
          return n3 = color, threeChannelSpaceSeparated(e3, normalize_HWB_ChannelValues, xe.HWB, [], n3);
        case "lab":
          return lab(e3, color);
        case "lch":
          return lch(e3, color);
        case "oklab":
          return oklab(e3, color);
        case "oklch":
          return oklch(e3, color);
        case "color":
          return color$1(e3, color);
        case "color-mix":
          return colorMix(e3, color);
        case "contrast-color":
          return contrastColor(e3, color);
        case "alpha":
          return alpha(e3, color);
      }
    }
    var n3;
    if (isTokenNode(e3)) {
      if (isTokenHash(e3.value)) return hex(e3.value);
      if (isTokenIdent(e3.value)) {
        const n4 = namedColor(e3.value[4].value);
        return false !== n4 ? n4 : "transparent" === toLowerCaseAZ2(e3.value[4].value) && { colorNotation: xe.RGB, channels: [0, 0, 0], alpha: 0, syntaxFlags: /* @__PURE__ */ new Set([De.ColorKeyword]) };
      }
    }
    return false;
  }

  // input-color.js
  var toXYZ = {
    "a98-rgb": a98_RGB_to_XYZ_D65,
    hsl: HSL_to_XYZ_D65,
    hwb: HWB_to_XYZ_D65,
    lab: Lab_to_XYZ_D65,
    lch: LCH_to_XYZ_D65,
    oklab: OKLab_to_XYZ_D65,
    oklch: OKLCH_to_XYZ_D65,
    "display-p3": P3_to_XYZ_D65,
    "display-p3-linear": lin_P3_to_XYZ_D65,
    "srgb-linear": lin_sRGB_to_XYZ_D65,
    "prophoto-rgb": ProPhoto_RGB_to_XYZ_D65,
    rec2020: rec_2020_to_XYZ_D65,
    "xyz-d50": XYZ_D50_to_XYZ_D65,
    "xyz-d65": (value) => value
  };
  var systemColors = {
    currentcolor: "#000000",
    accentcolor: "#007aff",
    accentcolortext: "#000000",
    activetext: "#ff0000",
    buttonborder: "#ffffff",
    buttonface: "#c0c0c0",
    buttontext: "#000000d8",
    canvas: "#ffffff",
    canvastext: "#000000",
    field: "#ffffff",
    fieldtext: "#000000d8",
    graytext: "#0000003f",
    highlight: "#80bcfe99",
    highlighttext: "#000000",
    linktext: "#0000ee",
    mark: "#ffff00",
    marktext: "#000000",
    selecteditem: "#c0c0c0",
    selecteditemtext: "#000000",
    visitedtext: "#551a8b"
  };
  var byte = (value) => {
    const scaled = Math.fround(Math.fround(value) * 255);
    return !Number.isFinite(scaled) || scaled >= 2 ** 63 ? 0 : Math.max(0, Math.min(255, Math.round(scaled)));
  };
  var decimal = (value) => String(Number((value / 255).toFixed(6)));
  var component = (value) => {
    value = Math.fround(value);
    if (Number.isNaN(value)) return "0";
    if (!Number.isFinite(value)) return value < 0 ? "calc(-infinity)" : "calc(infinity)";
    if (Number.isInteger(value)) return BigInt(value).toString();
    return String(Number(value.toFixed(6)));
  };
  function sanitize(value, alpha2, displayP3 = false) {
    const tokens = tokenize({ css: value }).flatMap((token) => {
      const name = token[0] === "ident-token" ? token[4].value.toLowerCase() : "";
      const replacement = Object.hasOwn(systemColors, name) && systemColors[name];
      return replacement ? tokenize({ css: replacement }).slice(0, -1) : [token];
    });
    const nodes = parseListOfComponentValues(tokens).filter((node2) => !isWhiteSpaceOrCommentNode(node2));
    const node = nodes.length === 1 && nodes[0];
    const parsed = node && color(node);
    let rgba = [0, 0, 0, 255];
    let p3 = [0, 0, 0], opacity = 1;
    if (parsed && typeof parsed.alpha === "number") {
      const channels = parsed.channels.map((value2) => Number.isNaN(value2) ? 0 : Math.fround(value2));
      const convert = toXYZ[parsed.colorNotation];
      const rgb2 = parsed.colorNotation === "hsl" ? HSL_to_sRGB(channels) : parsed.colorNotation === "hwb" ? HWB_to_sRGB(channels) : convert ? XYZ_D65_to_sRGB(convert(channels)) : channels;
      const byteColor = ["rgb", "hex", "hsl", "hwb"].includes(parsed.colorNotation) && !parsed.syntaxFlags.has("relative-color-syntax") && !parsed.syntaxFlags.has("color-mix");
      rgba = [
        ...rgb2.map((value2) => byte(byteColor ? Math.max(0, Math.min(1, value2)) : value2)),
        byte(Number.isNaN(parsed.alpha) ? 0 : parsed.alpha)
      ];
      if (displayP3) {
        opacity = Number.isNaN(parsed.alpha) ? 0 : Math.fround(parsed.alpha);
        if (byteColor) {
          p3 = XYZ_D65_to_P3(sRGB_to_XYZ_D65(rgba.slice(0, 3).map((value2) => value2 / 255)));
          opacity = rgba[3] / 255;
        } else if (parsed.colorNotation === "display-p3") p3 = channels;
        else p3 = XYZ_D65_to_P3(["hsl", "hwb"].includes(parsed.colorNotation) ? sRGB_to_XYZ_D65(rgb2) : convert ? convert(channels) : sRGB_to_XYZ_D65(channels));
      }
    }
    if (displayP3) return "color(display-p3 " + p3.map(component).join(" ") + (!alpha2 || opacity === 1 ? "" : " / " + component(opacity)) + ")";
    if (!alpha2) return "#" + rgba.slice(0, 3).map((value2) => value2.toString(16).padStart(2, "0")).join("");
    return "color(srgb " + rgba.slice(0, 3).map(decimal).join(" ") + (rgba[3] === 255 ? "" : " / " + decimal(rgba[3])) + ")";
  }
  return __toCommonJS(input_color_exports);
})();
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
/*! Bundled license information:

@csstools/color-helpers/dist/index.mjs:
  (**
   * Bradford chromatic adaptation from D50 to D65
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Bradford chromatic adaptation from D65 to D50
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see http://www.brucelindbloom.com/index.html?Eqn_ChromAdapt.html
   *)
  (**
   * @param {number} hue - Hue as degrees 0..360
   * @param {number} sat - Saturation as percentage 0..100
   * @param {number} light - Lightness as percentage 0..100
   * @return {number[]} Array of sRGB components; in-gamut colors in range [0..1]
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/hslToRgb.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see https://github.com/w3c/csswg-drafts/blob/main/css-color-4/hslToRgb.js
   *)
  (**
   * @param {number} hue -  Hue as degrees 0..360
   * @param {number} white -  Whiteness as percentage 0..100
   * @param {number} black -  Blackness as percentage 0..100
   * @return {number[]} Array of RGB components 0..1
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/hwbToRgb.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see https://github.com/w3c/csswg-drafts/blob/main/css-color-4/hwbToRgb.js
   *)
  (**
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert Lab to D50-adapted XYZ
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
   *)
  (**
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js
   *)
  (**
   * Given OKLab, convert to XYZ relative to D65
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js
   *)
  (**
   * Assuming XYZ is relative to D50, convert to CIE Lab
   * from CIE standard, which now defines these as a rational fraction
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *
   * XYZ <-> LMS matrices recalculated for consistent reference white
   * @see https://github.com/w3c/csswg-drafts/issues/6642#issuecomment-943521484
   *)
  (**
   * Convert XYZ to linear-light rec2020
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert XYZ to linear-light P3
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert D50 XYZ to linear-light prophoto-rgb
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
   *)
  (**
   * Convert XYZ to linear-light a98-rgb
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of linear-light rec2020 RGB  in the range 0.0-1.0
   * to gamma corrected form ITU-R BT.2020-2 p.4
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of linear-light sRGB values in the range 0.0-1.0 to gamma corrected form
   * Extended transfer function:
   *  For negative values, linear portion extends on reflection
   *  of axis, then uses reflected pow below that
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see https://en.wikipedia.org/wiki/SRGB
   *)
  (**
   * Convert an array of linear-light display-p3 RGB in the range 0.0-1.0
   * to gamma corrected form
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of linear-light prophoto-rgb in the range 0.0-1.0
   * to gamma corrected form.
   * Transfer curve is gamma 1.8 with a small linear portion.
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of linear-light a98-rgb in the range 0.0-1.0
   * to gamma corrected form. Negative values are also now accepted
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of rec2020 RGB values in the range 0.0 - 1.0
   * to linear light (un-companded) form.
   * ITU-R BT.2020-2 p.4
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of linear-light rec2020 values to CIE XYZ
   * using  D65 (no chromatic adaptation)
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
   *)
  (**
   * Convert an array of of sRGB values where in-gamut values are in the range
   * [0 - 1] to linear light (un-companded) form.
   * Extended transfer function:
   *  For negative values, linear portion is extended on reflection of axis,
   *  then reflected power function is used.
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see https://en.wikipedia.org/wiki/SRGB
   *)
  (**
   * Convert an array of display-p3 RGB values in the range 0.0 - 1.0
   * to linear light (un-companded) form.
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of linear-light display-p3 values to CIE XYZ
   * using D65 (no chromatic adaptation)
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
   *)
  (**
   * Convert an array of prophoto-rgb values where in-gamut Colors are in the
   * range [0.0 - 1.0] to linear light (un-companded) form. Transfer curve is
   * gamma 1.8 with a small linear portion. Extended transfer function
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of linear-light prophoto-rgb values to CIE D50 XYZ.
   * Matrix cannot be expressed in rational form, but is calculated to 64 bit accuracy.
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see see https://github.com/w3c/csswg-drafts/issues/7675
   *)
  (**
   * Convert an array of a98-rgb values in the range 0.0 - 1.0
   * to linear light (un-companded) form. Negative values are also now accepted
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of linear-light a98-rgb values to CIE XYZ
   * http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
   * has greater numerical precision than section 4.3.5.3 of
   * https://www.adobe.com/digitalimag/pdfs/AdobeRGB1998.pdf
   * but the values below were calculated from first principles
   * from the chromaticity coordinates of R G B W
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
   * @see https://www.adobe.com/digitalimag/pdfs/AdobeRGB1998.pdf
   * @see https://github.com/w3c/csswg-drafts/blob/main/css-color-4/matrixmaker.html
   *)
  (**
   * Convert an array of linear-light sRGB values to CIE XYZ
   * using sRGB's own white, D65 (no chromatic adaptation)
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/conversions.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *)
  (**
   * Convert an array of gamma-corrected sRGB values in the 0.0 to 1.0 range to HSL.
   *
   * @param {Color} RGB [r, g, b]
   * - Red component 0..1
   * - Green component 0..1
   * - Blue component 0..1
   * @return {number[]} Array of HSL values: Hue as degrees 0..360, Saturation and Lightness as percentages 0..100
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/utilities.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   *
   * @see https://github.com/w3c/csswg-drafts/blob/main/css-color-4/better-rgbToHsl.js
   *)

@csstools/color-helpers/dist/index.mjs:
  (**
   * @description Calculate deltaE OK which is the simple root sum of squares
   * @param {number[]} reference - Array of OKLab values: L as 0..1, a and b as -1..1
   * @param {number[]} sample - Array of OKLab values: L as 0..1, a and b as -1..1
   * @return {number} How different a color sample is from reference
   *
   * @license W3C https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document
   * @copyright This software or document includes material copied from or derived from https://github.com/w3c/csswg-drafts/blob/main/css-color-4/deltaEOK.js. Copyright © 2022 W3C® (MIT, ERCIM, Keio, Beihang).
   * @see https://github.com/w3c/csswg-drafts/blob/main/css-color-4/deltaEOK.js
   *)

@csstools/color-helpers/dist/index.mjs:
  (**
   * @license MIT https://github.com/facelessuser/coloraide/blob/main/LICENSE.md
   *)
*/
