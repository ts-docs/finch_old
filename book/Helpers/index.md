
# Helpers

Finch provides 4 very useful built-in helpers:

- `if`
- `each`
- `template`
- `js`

With these four helpers, you get iteration, logic, inheritance and customized behaviour. You can also create custom helpers, though the API right now is very limited.

## Custom Helpers

Registering custom helpers is done with the `addHelper` function. Currently, all arguments and the body come pre-compiled, which could be a performance concern, you may not want to compile the body if a condition is met, for example. In the future, you'll have to call a function to compile the arguments and the body. Custom helpers also cannot access followup blocks currently.

```js
Finch.addHelper("helperName", (args, body) => {
    const arr = args[0];
    const delimiter = args[1];
    return arr.join(delimiter || ", ");
});
```
