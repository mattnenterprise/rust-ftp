FROM i386/ubuntu:latest

RUN apt-get update -qq &&\
    apt-get install -yqq vsftpd

RUN mkdir -p /var/run/vsftpd/empty &&\
    useradd -s /bin/bash -d /home/ftp -m -c "Doe ftp user" -g ftp Doe &&\
    echo "Doe:mumble"| chpasswd &&\
    echo "listen=yes" > /etc/vsftpd.conf &&\
    echo "anon_root=/home/ftp" >> /etc/vsftpd.conf &&\
    echo "local_enable=yes" >> /etc/vsftpd.conf &&\
    echo "local_umask=022" >> /etc/vsftpd.conf &&\
    echo "pasv_enable=YES" >> /etc/vsftpd.conf &&\
    echo "pasv_min_port=65000" >> /etc/vsftpd.conf &&\
    echo "pasv_max_port=65010" >> /etc/vsftpd.conf &&\
    echo "write_enable=yes" >> /etc/vsftpd.conf &&\
    echo "log_ftp_protocol=yes" >> /etc/vsftpd.conf

CMD ["/bin/bash"]
