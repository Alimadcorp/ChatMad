import React, { useEffect, useState } from 'react';
import { DbConnection, type ErrorContext, type EventContext, Message, User, Channel } from './module_bindings';
import { Identity } from '@clockworklabs/spacetimedb-sdk';
import './App.css';

export type PrettyMessage = {
  senderName: string;
  text: string;
};
export type Chat = {
  username: string;
  lastMessage: string;
  notifications: bigint;
};

function App() {
  const [newName, setNewName] = useState('');
  const [settingName, setSettingName] = useState(false);
  const [systemMessage, setSystemMessage] = useState('');
  const [newMessage, setNewMessage] = useState('');
  const [connected, setConnected] = useState<boolean>(false);
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [conn, setConn] = useState<DbConnection | null>(null);

  useEffect(() => {
    const subscribeToQueries = (conn: DbConnection, queries: string[]) => {
      conn
        ?.subscriptionBuilder()
        .onApplied(() => {
          console.log('SDK client cache initialized.');
        })
        .subscribe(queries);
    };

    const onConnect = (
      conn: DbConnection,
      identity: Identity,
      token: string
    ) => {
      setIdentity(identity);
      setConnected(true);
      localStorage.setItem('auth_token', token);
      console.log(
        'Connected to SpacetimeDB with identity:',
        identity.toHexString()
      );
      conn.reducers.onSendMessage(() => {
        console.log('Message sent.');
      });

      subscribeToQueries(conn, ['SELECT * FROM message WHERE channel="global"', 'SELECT * FROM user']);
    };

    const onDisconnect = () => {
      console.log('Disconnected from SpacetimeDB');
      setConnected(false);
    };

    const onConnectError = (_ctx: ErrorContext, err: Error) => {
      console.log('Error connecting to SpacetimeDB:', err);
    };

    setConn(
      DbConnection.builder()
        .withUri('wss://maincloud.spacetimedb.com')
        .withModuleName('chatmad')
        .withToken(localStorage.getItem('auth_token') || '')
        .onConnect(onConnect)
        .onDisconnect(onDisconnect)
        .onConnectError(onConnectError)
        .build()
    );
  }, []);

const prettyMessages: PrettyMessage[] = [];
const chats: Chat[] = [];

  const name = '';

  const onSubmitNewName = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setSettingName(false);
    // TODO: Call `setName` reducer
  };

  const onMessageSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setNewMessage('');
    // TODO: Call `sendMessage` reducer
  };

  return (
    <div className="App">
      <div className='Chats'>
        <h1 className='title'>ChatMad</h1>
        <h3>Chats</h3>
        {chats.length < 1 && <p>No messages</p>}
        <div>
          {chats.map((chat, key) => (
            <div key={key}>
              <p><b>{chat.username}</b></p>
              <p>{chat.lastMessage}</p>
            </div>
          ))}
        </div>
      </div>
      <div className="Messages">
        <h1>Messages</h1>
        {prettyMessages.length < 1 && <p>No messages</p>}
        <div>
          {prettyMessages.map((message, key) => (
            <div key={key}>
              <p><b>{message.senderName}</b></p>
              <p>{message.text}</p>
            </div>
          ))}
        </div>
        <div className="NewMessage">
        <form onSubmit={onMessageSubmit}>
          <textarea
            value={newMessage}
            onChange={e => setNewMessage(e.target.value)}
          ></textarea>
          <button type="submit">Send</button>
        </form>
      </div>
      </div>
      <div className='UserPanel'>
        <h1>Profile</h1>
        {!settingName ? (
          <>
            <p>{name}</p>
            <button
              onClick={() => {
                setSettingName(true);
                setNewName(name);
              }}
            >
              Edit Name
            </button>
          </>
        ) : (
          <form onSubmit={onSubmitNewName}>
            <input
              type="text"
              value={newName}
              onChange={e => setNewName(e.target.value)}
            />
            <button type="submit">Submit</button>
          </form>
        )}
      </div>
    </div>
  );
}

export default App;