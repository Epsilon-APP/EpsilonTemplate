FROM openjdk:8-jdk-alpine3.7

ARG TEMPLATE_NAME
ENV TEMPLATE_NAME $TEMPLATE_NAME

ARG API_HOST
ENV API_HOST $API_HOST

WORKDIR /data/${TEMPLATE_NAME}

ADD http://${API_HOST}/templates/${TEMPLATE_NAME}/zip template.zip

RUN apk add zip
RUN unzip template.zip
RUN rm template.zip

CMD java -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1 -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -Dusing.aikars.flags=https://mcflags.emc.gs -Daikars.new.flags=true -jar -Dcom.mojang.eula.agree=true -DIknowWhatImDoingISwear server.jar nogui