FROM openjdk:15-jdk-alpine

ARG TEMPLATE_NAME
ENV TEMPLATE_NAME $TEMPLATE_NAME

ARG DEFAULT_MAP_NAME
ENV DEFAULT_MAP_NAME $DEFAULT_MAP_NAME

ARG API_HOST
ENV API_HOST $API_HOST

WORKDIR /data/${TEMPLATE_NAME}

RUN apk add zip

CMD wget -O template.zip http://${API_HOST}:8000/templates/${TEMPLATE_NAME}/zip && unzip template.zip && rm template.zip && clear && java -XX:MaxRAMPercentage=75 -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1 -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -Dusing.aikars.flags=https://mcflags.emc.gs/ -Daikars.new.flags=true -jar -Dcom.mojang.eula.agree=true -DIknowWhatImDoingISwear server.jar --level-name ${DEFAULT_MAP_NAME} nogui --noconsole
