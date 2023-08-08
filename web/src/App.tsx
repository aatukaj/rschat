import { join } from 'path';
import React, { useEffect, useState } from 'react';
import useWebSocket, { ReadyState, SendMessage } from "react-use-websocket";
import { json } from 'stream/consumers';



// https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
enum Color {
  Black = "Black",
  Red = "Red",
  Green = "Green",
  Yellow = "Yellow",
  Blue = "Blue",
  Magenta = "Magenta",
  Cyan = "Cyan",
  Gray = "Gray",
  DarkGray = "DarkGray",
  LightRed = "LightRed",
  LightGreen = "LightGreen",
  LightYellow = "LightYellow",
  LightBlue = "LightBlue",
  LightMagenta = "LightMagenta",
  LightCyan = "LightCyan",
  White = "White",
}
enum AppState {
  Login,
  Chat,
}


interface NewUserSet {
  user_name: string,
  color: Color,
}
interface Message {
  user_name: String,
  content: String,
  color: Color,
}

function MessageList(props: { messages: Message[] }) {
  return (
    <ul className="ml-2">
      {props.messages.map((msg, i) => <li key={i}>
        <span className={`${colorToClassText[msg.color]}`}><b>{msg.user_name}:</b></span> {msg.content}
      </li>)}
    </ul>
  )
}

const colorToClass = {
  [Color.Black]: "bg-zinc-950",
  [Color.Red]: "bg-red-600",
  [Color.Green]: "bg-green-500",
  [Color.Yellow]: "bg-yellow-400",
  [Color.Blue]: "bg-blue-600",
  [Color.Magenta]: "bg-fuchsia-600",
  [Color.Cyan]: "bg-cyan-500",
  [Color.Gray]: "bg-zinc-500",
  [Color.DarkGray]: "bg-zinc-900",
  [Color.LightRed]: "bg-red-400",
  [Color.LightGreen]: "bg-green-300",
  [Color.LightYellow]: "bg-yellow-200",
  [Color.LightBlue]: "bg-blue-300",
  [Color.LightMagenta]: "bg-fuchsia-300",
  [Color.LightCyan]: "bg-cyan-200",
  [Color.White]: "bg-white",
}
const colorToClassText = {
  [Color.Black]: "text-zinc-950",
  [Color.Red]: "text-red-600",
  [Color.Green]: "text-green-500",
  [Color.Yellow]: "text-yellow-400",
  [Color.Blue]: "text-blue-600",
  [Color.Magenta]: "text-fuchsia-600",
  [Color.Cyan]: "text-cyan-500",
  [Color.Gray]: "text-zinc-500",
  [Color.DarkGray]: "text-zinc-900",
  [Color.LightRed]: "text-red-400",
  [Color.LightGreen]: "text-green-300",
  [Color.LightYellow]: "text-yellow-200",
  [Color.LightBlue]: "text-blue-300",
  [Color.LightMagenta]: "text-fuchsia-300",
  [Color.LightCyan]: "text-cyan-200",
  [Color.White]: "text-white",
}


function Login(props: { handleSubmit: (userName: string, userColor: Color) => void }) {
  const [userName, setUserName] = useState("");
  const [userColor, setUserColor] = useState(Color.White);

  return (
    <>
      <form className="bg-zinc-900 rounded-lg p-5 shadow-xl flex flex-col min-h-[33%]" onSubmit={(e) => {
        e.preventDefault(); props.handleSubmit(userName, userColor);
      }}>

        <input
          type="text"
          className="rounded-md bg-zinc-800 p-1 shadow-xl placeholder:italic placeholder:text-zinc-600 h-10 flex-none"
          placeholder="Username" value={userName} maxLength={15} onChange={(e) => setUserName(e.target.value)}>
        </input>
        <div className="grid grid-cols-4 items-stretch gap-x-1 gap-y-1 mt-2 basis-1/2 flex-grow">
          {(Object.keys(Color) as Array<Color>).map((color, i) =>
            <input
              key={i} type="radio"
              className={`appearance-none rounded-md border-2 border-zinc-800 m-0 min-h-[16px] ring-transparent ${colorToClass[color]} ring-1 focus:ring-white`}
              value={color} onChange={(e => setUserColor(e.target.value as Color))} name="color"
            />
          )}
        </div>
        <input
          type="submit" value="Join"
          className="text-center font-bold w-full bg-blue-500 hover:bg-blue-600 rounded-md h-10 flex-none mt-2 shadow-md shadow-blue-500/50 hover:shadow-blue-600/50">
        </input>
      </form></>
  );
}




function App() {
  const [appState, setAppState] = useState(AppState.Login);

  const [messagesHistory, setMessagesHistory] = useState(Array<Message>);

  const [curMessage, setCurMessage] = useState("");

  const { sendMessage, lastMessage, readyState } = useWebSocket('ws://127.0.0.1:8080');

  useEffect(() => {
    if (lastMessage !== null) {
      let data: Blob = lastMessage.data;
      data.text().then((text) => setMessagesHistory((prev) => prev.concat(JSON.parse(text))));

    }
  }, [lastMessage, setMessagesHistory]);


  return (
    <>
      {
        appState === AppState.Login ?
          <div className="flex flex-col justify-center h-screen items-center">
            <Login
              handleSubmit={(userName, userColor) => { setAppState(AppState.Chat); sendMessage(JSON.stringify({ user_name: userName, color: userColor })) }}
            />
          </div>
          :
          <div className="h-screen p-5 ">
            <div className="h-full flex items-stretch flex-col justify-end p-5 bg-zinc-900 rounded-lg gap-3">
              <MessageList messages={messagesHistory} />
              <form onSubmit={(e) => {
                e.preventDefault();
                sendMessage(curMessage)
                setCurMessage("");
              }}>
                <input
                  className="rounded-md bg-zinc-800 p-1 shadow-xl placeholder:italic placeholder:text-zinc-600 h-10 flex-none w-full"
                  type="text" value={curMessage} placeholder="Message"
                  onChange={(e) => setCurMessage(e.target.value)}>
                </input>
                <input type="submit" hidden />
              </form>
            </div>
          </div>

      }
    </>




  );
}

export default App;


