FROM i386/ubuntu:latest

RUN apt-get update -qq &&\
    apt-get install -yqq vsftpd

RUN mkdir -p /var/run/vsftpd/empty &&\
    useradd -s /bin/bash -d /home/ftp -m -c "Doe ftp user" -g ftp Doe &&\
    echo "Doe:mumble"| chpasswd &&\
    echo "listen=yes\n\
anon_root=/home/ftp\n\
local_enable=yes\n\
local_umask=022\n\
pasv_enable=YES\n\
pasv_min_port=65000\n\
pasv_max_port=65010\n\
write_enable=yes\n\
log_ftp_protocol=yes" > /etc/vsftpd.conf &&\
    echo "/etc/init.d/vsftpd start" | tee -a /etc/bash.bashrc

CMD ["/bin/bash"]
