let importjs = (str) => import(/* webpackIgnore: true */str)

let importDanfoJS = () => import("../../../../../../../../website/node_modules/danfojs/dist/index.js")

let importTFJS_VIS = () => import("../../../../../../../../website/node_modules/@tensorflow/tfjs-vis/dist/index.js")

export {
  importjs,
  importDanfoJS,
  importTFJS_VIS
}
