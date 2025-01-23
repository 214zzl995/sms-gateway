import { writable } from 'svelte/store';

export const devices = writable([
  {
    id: 1,
    name: "Modem-01",
    status: "online",
  },
  {
    id: 2,
    name: "Modem-02",
    status: "offline",
  },
  {
    id: 3,
    name: "Modem-03",
    status: "maintenance",
  },
]);

export const messages = writable([
  { id: 1, content: "系统启动成功", timestamp: "09:30:45" },
  { id: 2, content: "信号强度: 92%", timestamp: "09:31:10" },
]);