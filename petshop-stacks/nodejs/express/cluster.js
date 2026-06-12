const cluster = require('cluster');

const workers = parseInt(process.env.WEB_CONCURRENCY || '1', 10);

if (workers > 1 && cluster.isPrimary) {
  for (let i = 0; i < workers; i += 1) {
    cluster.fork();
  }
  cluster.on('exit', (worker, code, signal) => {
    console.log(`Worker ${worker.process.pid} died (${signal || code}), respawning`);
    cluster.fork();
  });
} else {
  require('./index');
}
