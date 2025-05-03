/**
 * @param {string | number | Date} date
 */
export function formatDate(date) {
    const input = new Date(date);
    const now = new Date() ;

    const isSameDay = input.toDateString() === now.toDateString();

    const diffTime = now.getTime() - input.getTime();
    const diffDays = diffTime / (1000 * 60 * 60 * 24);

    if (isSameDay) {
        return input.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (diffDays < 7) {
        return input.toLocaleDateString(undefined, { weekday: 'long' });
    } else {
        return input.toLocaleDateString(undefined, { year: 'numeric', month: '2-digit', day: '2-digit' });
    }
}
