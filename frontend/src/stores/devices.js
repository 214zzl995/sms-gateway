import { writable } from 'svelte/store';
import { apiClient } from '../js/api';

export const devices = writable([]);

export const initDeices = () => {
    getDevices()
}

export const refreshDevices = () => {
    getDevices()
}

const getDevices = () => {
    apiClient.getAllModems().then(
        (res) => {
            devices.set(res.data)
        }
    )

    // const test = [
    //     {
    //         "modem_model": {
    //             "data": {
    //                 "model": "EC200A"
    //             },
    //             "error": null
    //         },
    //         "name": "测试卡片1",
    //         "network_registration": {
    //             "data": {
    //                 "cell_id": null,
    //                 "location_area_code": "1",
    //                 "status": "0"
    //             },
    //             "error": null
    //         },
    //         "operator": {
    //             "data": {
    //                 "operator_id": "0",
    //                 "operator_name": "CHN-CT",
    //                 "registration_status": "0"
    //             },
    //             "error": null
    //         },
    //         "signal_quality": {
    //             "data": {
    //                 "ber": 99,
    //                 "rssi": 31
    //             },
    //             "error": null
    //         }
    //     },

    //     {
    //         "modem_model": {
    //             "data": {
    //                 "model": "EC200B"
    //             },
    //             "error": null
    //         },
    //         "name": "测试卡片2",
    //         "network_registration": {
    //             "data": {
    //                 "cell_id": null,
    //                 "location_area_code": "1",
    //                 "status": "0"
    //             },
    //             "error": null
    //         },
    //         "operator": {
    //             "data": {
    //                 "operator_id": "0",
    //                 "operator_name": "CHN-CT",
    //                 "registration_status": "0"
    //             },
    //             "error": null
    //         },
    //         "signal_quality": {
    //             "data": {
    //                 "ber": 99,
    //                 "rssi": 31
    //             },
    //             "error": null
    //         }
    //     }
    // ];

    // devices.set(test)
}
