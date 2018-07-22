let Exonum = require('exonum-client')

console.log('Hello, Exonum!');

const pair = Exonum.keyPair()
console.log(pair)

// curl -H "Content-Type: application/json" -X POST -d @create_rate.json http://127.0.0.1:30001/api/services/rates/v1/rate

