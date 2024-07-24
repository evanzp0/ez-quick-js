import * as _G from '_G'
var g = new _G.Print(1);
g.PrintTestFunc();
g.val = 3;
console.log(g.val);

console.log("-------------");

import {Print} from '_G';
var g = new Print(2);
g.PrintTestFunc();
g.val = 4;
console.log(g.val);