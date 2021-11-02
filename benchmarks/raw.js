
const Finch = require("../");
const Handlebars = require("handlebars");
const Eta = require("eta");
const { performance } = require("perf_hooks"); 


const before_p_1 = performance.now();
Finch.addTemplate("test_1", `
<div>
{{user.name}}
{{#each user.numbers num}}
    {{num}}
{{/}}
</div>
`);
console.log(`Parsing finch took: ${performance.now() - before_p_1}`);

const before_p_2 = performance.now();
const testhb = Handlebars.compile(`
<div>
{{user.name}}
{{#each user.numbers}}
    {{this}}
{{/each}}
</div>
`);
console.log(`Parsing handlebars took: ${performance.now() - before_p_2}`);

const before_p_3 = performance.now();
const testejs = Eta.compile(`
<div>
<%= it.user.name %>
<% for (const num of it.user.numbers) { %>
    <%= num %>
<% } %>
</div>
`);
console.log(`Parsing eta took: ${performance.now() - before_p_3}`);


const before3 = performance.now();
for (let i=0; i < 10000; i++) {
    testejs({user: {name: "google", numbers: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]}}, Eta.getConfig());
}
console.log(`Eta took: ${performance.now() - before3}MS`);

const before1 = performance.now();
for (let i=0; i < 10000; i++) {
    Finch.compile("test_1", {user: {name: "google", numbers: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]}});
}
console.log(`Finch took: ${performance.now() - before1}MS`);

const before2 = performance.now();
for (let i=0; i < 10000; i++) {
    testhb({user: {name: "google", numbers: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]}});
}
console.log(`Handlebars took: ${performance.now() - before2}MS`);
