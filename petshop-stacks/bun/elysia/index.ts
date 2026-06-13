import { buildApp } from './app';

const app = buildApp();
const port = parseInt(process.env.PORT || '8080', 10);

app.listen(port, () => {
  console.log(`Listening on :${port}`);
});
