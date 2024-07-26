import {Print} from '_G';
var g = new Print(2);
g.PrintTestFunc();
g.val = 4;

let ret_val = 40;

function add_one(val) {
    return val + 1;
}

export default "evan";
export { ret_val, add_one };