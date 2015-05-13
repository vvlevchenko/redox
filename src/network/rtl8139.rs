use common::debug::*;
use common::memory::*;
use common::pio::*;

use network::network::*;

pub struct RTL8139 {
    pub base: usize,
    pub memory_mapped: bool
}

static mut RTL8139_TX: u16 = 0;

impl NetworkDevice for RTL8139 {
    unsafe fn send(&self, addr: usize, len: usize){
        if cfg!(debug_network){
            d("RTL8139 send ");
            dd(RTL8139_TX as usize);
            dl();
        }

        let base = self.base as u16;

        outd(base + 0x20 + RTL8139_TX*4, addr as u32);
        outd(base + 0x10 + RTL8139_TX*4, len as u32 & 0x1FFF);

        while ind(base + 0x10 + RTL8139_TX*4) & (1 << 13) == 0 {
            //Waiting for move out of memory
        }

        RTL8139_TX = (RTL8139_TX + 1) % 4;
    }
}

impl RTL8139 {
    pub unsafe fn handle(&self){
        if cfg!(debug_network){
            d("RTL8139 handle");
        }

        let base = self.base as u16;

        let receive_buffer = ind(base + 0x30) as usize;
        let mut capr = (inw(base + 0x38) + 16) as usize;
        let cbr = inw(base + 0x3A) as usize;
        while capr != cbr {
            let frame_addr = receive_buffer + capr + 4;
            let frame_len = *((receive_buffer + capr + 2) as *const u16) as usize;

            if cfg!(debug_network){
                d(" CAPR ");
                dd(capr);
                d(" CBR ");
                dd(cbr);

                d(" len ");
                dd(frame_len);
                dl();
            }

            network_frame(self, frame_addr, frame_len);

            capr = capr + frame_len + 4;
            capr = (capr + 3) & (0xFFFFFFFF - 3);
            if capr >= 8192 {
                capr -= 8192
            }

            outw(base + 0x38, (capr as u16) - 16);
        }

        outw(base + 0x3E, 0x0001);
    }

    pub unsafe fn init(&self){
        d("RTL8139 on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        dl();

        let base = self.base as u16;

        outb(base + 0x52, 0x00);

        outb(base + 0x37, 0x10);
        while inb(base + 0x37) & 0x10 != 0 {
        }

        RTL8139_TX = 0;

        let receive_buffer = alloc(10240);
        outd(base + 0x30, receive_buffer as u32);
        outw(base + 0x38, 0);
        outw(base + 0x3A, 0);

        outw(base + 0x3C, 0x0001);

        outd(base + 0x44, 0xf | (1 << 7));

        outb(base + 0x37, 0x0C);
    }
}