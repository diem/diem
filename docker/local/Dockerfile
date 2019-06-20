FROM rust:latest

RUN git version

RUN git clone https://github.com/libra/libra

RUN cd libra && \
        sed -i 's/sudo/ /g' scripts/dev_setup.sh &&\
        sed -i 's/apt-get/apt/g' scripts/dev_setup.sh &&\
        yes | ./scripts/dev_setup.sh
