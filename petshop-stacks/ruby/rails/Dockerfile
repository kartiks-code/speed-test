FROM ruby:3.4-alpine

RUN apk add --no-cache build-base postgresql-dev tzdata yaml-dev

WORKDIR /app

COPY . .

RUN gem install bundler --no-document && bundle install

RUN chmod +x bin/rails

COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
ENTRYPOINT ["docker-entrypoint.sh"]

EXPOSE 8080

CMD ["bin/rails", "server", "-b", "0.0.0.0", "-p", "8080"]
