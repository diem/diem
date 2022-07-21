const getCookie = name => {
  const cookieRow = document.cookie
    .split('; ')
    .find(row => row.startsWith(`${name}=`));

  if (!cookieRow) {
    return;
  }

  return cookieRow.split('=')[1];
};

export default getCookie;
