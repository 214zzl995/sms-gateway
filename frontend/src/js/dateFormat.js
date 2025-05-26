/**
 * @param {string | number | Date} date
 * @returns {string}
 */
export function formatDate(date) {
    const input = new Date(date);
    const now = new Date();

    const isSameDay = input.toDateString() === now.toDateString();

    const inputIso = getISOWeekAndYear(input);
    const nowIso = getISOWeekAndYear(now);

    const isInThisWeek = inputIso.year === nowIso.year && inputIso.week === nowIso.week;

    if (isSameDay) {
        return input.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
    } else if (isInThisWeek) {
        return input.toLocaleDateString(undefined, { weekday: 'long' });
    } else {
        return input.toLocaleDateString(undefined, { year: 'numeric', month: '2-digit', day: '2-digit' });
    }
}

export function formatTimeRange(start, end) {
    if (end == null) {
        end = new Date();
    }
    const startTime = new Date(start);
    const endTime = new Date(end);

    const now = new Date();
    const diffInMinutes = (endTime.getTime() - startTime.getTime()) / (1000 * 60);

    if (diffInMinutes < 10) {
        return '';
    }

    const year = now.getFullYear();
    const startYear = startTime.getFullYear();
    const startMonth = startTime.getMonth();
    const startDay = startTime.getDate();
    const startHour = startTime.getHours();
    const startMinute = startTime.getMinutes();

    if (startTime.toDateString() === now.toDateString()) {
        return `${startHour}:${startMinute < 10 ? '0' + startMinute : startMinute}`;
    }

    const startWeekStart = getWeekStart(startTime).getTime();
    const nowWeekStart = getWeekStart(now).getTime();

    if (startWeekStart === nowWeekStart) {
        const daysOfWeek = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
        const weekDay = daysOfWeek[startTime.getDay()];
        return `${weekDay} ${startHour}:${startMinute < 10 ? '0' + startMinute : startMinute}`;
    }

    if (startYear === year) {
        return `${startMonth + 1}/${startDay} ${startHour}:${startMinute < 10 ? '0' + startMinute : startMinute}`;
    }

    return `${startYear}/${startMonth + 1}/${startDay} ${startHour}:${startMinute < 10 ? '0' + startMinute : startMinute}`;
}

const getWeekStart = (/** @type {string | number | Date} */ d) => {
    const temp = new Date(d);
    temp.setHours(0, 0, 0, 0);
    temp.setDate(temp.getDate() - temp.getDay());
    return temp;
}


const getISOWeekAndYear = (/** @type {string | number | Date} */ d) => {
    const dt = new Date(d);
    dt.setHours(0, 0, 0, 0);
    dt.setDate(dt.getDate() + 3 - (dt.getDay() + 6) % 7);
    const week1 = new Date(dt.getFullYear(), 0, 4);
    return {
        year: dt.getFullYear(),
        week: 1 + Math.round(((dt.getTime() - week1.getTime()) / 86400000 -
            3 + (week1.getDay() + 6) % 7) / 7)
    };
}

