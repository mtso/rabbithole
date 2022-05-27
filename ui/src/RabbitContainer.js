import { useEffect } from "react";
import { useSelector, useDispatch } from "react-redux";
import { setStatus, setRabbit } from "./rabbitSlice";

import Rabbit from "./Rabbit";
import RabbitBirthing from "./RabbitBirthing";

function createDelay(t) {
  return new Promise((resolve) => {
    setTimeout(() => resolve(), t);
  });
}

export default function RabbitContainer() {
  const { id, status, bodyColor, patchColor, eyeColor } = useSelector((state) => state.rabbit);
  const dispatch = useDispatch();

  useEffect(() => {
    if (id && status !== "birthed") {
      const handleFetch = () => {

      return fetch("/api3/rabbits/" + id).then((resp) => resp.json()).then((rabbit) => {
        const { status, body_color, patch_color, eye_color } = rabbit;
        console.log("handleFetch result", status, rabbit);

        dispatch(setRabbit({
          status,
          bodyColor: body_color,
          patchColor: patch_color,
          eyeColor: eye_color,
        }));

        if (status !== "birthed") {
          return createDelay(2000).then(() => handleFetch());
        } else {
          return Promise.resolve();
        }       
      }).catch(console.error);


      }

      handleFetch().catch(console.error);
    }
  }, [id, status, dispatch]);

  if (status === null) {
    return <div
      style={{
        fontSize: '2em',
        fontWeight: "bold",
        margin: "4em auto",
        textAlign: "center",
      }}
    >Loading...</div>
  }
  else if (status === "pending") {
    return <RabbitBirthing />
  }
  else if (status === "birthed") {
    return <Rabbit
      id={id}
      status={status}
      bodyColor={bodyColor}
      patchColor={patchColor}
      eyeColor={eyeColor}
    />
  } else {
    console.log("Invalid status", status);
    return <div>???</div>
  }
}
