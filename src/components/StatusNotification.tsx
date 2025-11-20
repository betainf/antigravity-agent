import React from 'react';

const StatusNotification = ({ status }) => {
  if (!status.message) return null;

  return (
    <div className={`fixed bottom-8 right-8 max-w-md z-50 animate-fade-in ${
      status.isError ? 'status-error' : 'status-success'
    }`}>
      {status.message}
    </div>
  );
};

export default StatusNotification;