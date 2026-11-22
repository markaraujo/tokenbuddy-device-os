#include "connectivity.h"
#include "driver/usb_serial_jtag.h"
#include "driver/usb_serial_jtag_vfs.h"
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
static char pending[769];
static unsigned used;
static bool overflow;
void link_init(void)
{
    usb_serial_jtag_driver_config_t cfg={.tx_buffer_size=4096,.rx_buffer_size=2048};
    ESP_ERROR_CHECK(usb_serial_jtag_driver_install(&cfg));
    usb_serial_jtag_vfs_use_driver();
    fcntl(fileno(stdout),F_SETFL,O_NONBLOCK);
}
bool link_read(char *line,unsigned capacity)
{
    char ch;
    /* Bound CPU time even when a sender floods bytes without a newline. */
    for (unsigned i=0;i<256 && usb_serial_jtag_read_bytes(&ch,1,0)==1;++i) {
        if (ch=='\r') continue;
        if (ch=='\n') {
            if (overflow || used>=capacity) snprintf(line,capacity,"{}");
            else {memcpy(line,pending,used); line[used]=0;}
            used=0; overflow=false; return true;
        }
        if ((unsigned char)ch<32 || used==sizeof(pending)-1) overflow=true;
        if (!overflow) pending[used++]=ch;
    }
    return false;
}
void link_write(const char *line)
{
    size_t size=strlen(line);
    if (size>1022) return;
    char frame[1024]; memcpy(frame,line,size);frame[size++]='\n';
    usb_serial_jtag_write_bytes(frame,size,pdMS_TO_TICKS(20));
}
