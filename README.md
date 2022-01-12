# Weblab interpreter

A lite Javascript interpreter written in Rust and compiled to WASM using wasm-pack. The project is in alpha stage, please use it with caution.

It is used for the execution of the cells of [Weblab](https://www.weblab.ai) notebooks.

## Usage

Evaluate a cell:

```javascript
eval_cell(str: String): Promise<String | HtmlElement>
```

Reset the environment:

```javascript
reset_envs()
```