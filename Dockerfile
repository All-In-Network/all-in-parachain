# Bundle static assets with nginx
FROM nginx:alpine as production

# Add your nginx.conf
COPY ./conf/rpc/nginx/rpc.all-in.app.conf.template /etc/nginx/conf.d/default.conf

# Expose port
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
