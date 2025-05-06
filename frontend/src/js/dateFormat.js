/**
 * @param {string | number | Date} date
 */
export function formatDate(date) {
    const input = new Date(date);
    const now = new Date() ;

    const isSameDay = input.toDateString() === now.toDateString();

    const startOfWeek = now.getDate() - now.getDay(); 
    const endOfWeek = startOfWeek + 6; 
    const isInThisWeek = input.getDate() >= startOfWeek && input.getDate() <= endOfWeek;

    if (isSameDay) {
        return input.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (isInThisWeek) {
        return input.toLocaleDateString(undefined, { weekday: 'long' });
    } else {
        return input.toLocaleDateString(undefined, { year: 'numeric', month: '2-digit', day: '2-digit' });
    }
}

/**
 * @param {string | number | Date} start
 * @param {string | number | Date} end
 */
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

    const diffInDays = Math.floor((now.getTime() - startTime.getTime()) / (1000 * 60 * 60 * 24));
    if (diffInDays <= 7) {
        const daysOfWeek = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
        const weekDay = daysOfWeek[startTime.getDay()];
        return `${weekDay} ${startHour}:${startMinute < 10 ? '0' + startMinute : startMinute}`;
    }


    if (startYear === year) {
        return `${startMonth + 1}/${startDay} ${startHour}:${startMinute < 10 ? '0' + startMinute : startMinute}`;
    }

    return `${startYear}/${startMonth + 1}/${startDay} ${startHour}:${startMinute < 10 ? '0' + startMinute : startMinute}`;
}

