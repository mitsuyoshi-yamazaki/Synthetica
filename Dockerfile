FROM node:24

WORKDIR /usr/src/synthetica
COPY ./synthetica .

# RUN yarn install