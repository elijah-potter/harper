FROM rust:latest as wasm-build

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

COPY . .

WORKDIR /usr/build/harper-wasm
RUN wasm-pack build --release

FROM node:slim as node-build

RUN mkdir -p /usr/build/
WORKDIR /usr/build/

RUN mkdir harper-wasm

COPY --from=wasm-build /usr/build/harper-wasm/pkg /usr/build/harper-wasm/pkg
COPY packages/web web
COPY demo.md .

WORKDIR /usr/build/web

RUN yarn install && yarn build

FROM node:slim

COPY --from=node-build /usr/build/web/build /usr/build/web/build
COPY --from=node-build /usr/build/web/package.json /usr/build/web/package.json 
COPY --from=node-build /usr/build/web/yarn.lock /usr/build/web/yarn.lock
COPY --from=node-build /usr/build/web/node_modules /usr/build/web/node_modules

WORKDIR /usr/build/web

RUN yarn install

ENV HOST=0.0.0.0
ENV PORT=3000

ENTRYPOINT ["node", "build"]
