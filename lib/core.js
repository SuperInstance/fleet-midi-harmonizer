function process(v, b=60) {
  return v.reduce((n,x) => [...n, x===1?n[n.length-1]+4:x===-1?n[n.length-1]-4:n[n.length-1]], [b]);
}
module.exports = { process };
if (require.main === module) console.log(process([1,0,-1,1,0,-1,1,1]));
