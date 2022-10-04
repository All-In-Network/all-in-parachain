FROM node:alpine

WORKDIR /app

# Clone the frontend repo
# ...

# ...


COPY package.json ./
COPY package-lock.json ./

RUN npm i

CMD ["npm", "run", "start"]
