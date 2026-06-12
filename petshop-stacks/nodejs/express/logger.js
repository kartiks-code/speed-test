const { transports, createLogger, format } = require('winston');

const isProduction = process.env.NODE_ENV === 'production';

const logger = createLogger({
  level: isProduction ? 'error' : 'info',
  format: format.combine(
    format.timestamp(),
    format.json(),
  ),
  defaultMeta: { service: 'user-service' },
  // In production, skip file transports and log only errors to the console.
  transports: isProduction
    ? [new transports.Console({ level: 'error' })]
    : [
      new transports.Console(),
      new transports.File({ filename: 'error.log', level: 'error', timestamp: true }),
      new transports.File({ filename: 'combined.log', timestamp: true }),
    ],
});

if (!isProduction) {
  logger.add(new transports.Console({ format: format.simple() }));
}

module.exports = logger;
