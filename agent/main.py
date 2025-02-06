import logging
import random
import re
import urllib
from datetime import datetime
from typing import Annotated

import aiohttp
import pytz
from dotenv import load_dotenv
from livekit.agents import (AutoSubscribe, JobContext, JobProcess,
                            WorkerOptions, cli, llm)
from livekit.agents.pipeline import AgentCallContext, VoicePipelineAgent
from livekit.plugins import deepgram, openai, silero

baseURLW = "http://192.168.10.14:8027/v1"
#baseURL = "http://192.168.10.14:8002/v1"
baseURL = "http://192.168.10.14:8880/v1"

load_dotenv()

logger = logging.getLogger("weather-demo")
logger.setLevel(logging.INFO)

import io
import logging
import wave

import requests
from dotenv import load_dotenv
from livekit.agents import stt, utils
from livekit.agents.utils import AudioBuffer
from pydub import AudioSegment

# class myClass(stt.STT):
#     def __init__(
#         self,
#     ):
#         super().__init__(
#             capabilities=stt.STTCapabilities(streaming=False, interim_results=False)
#         )
#
#     async def _recognize_impl(
#         self, buffer: AudioBuffer, *, language: str | None = None, conn_options: dict | None = None
#     ) -> stt.SpeechEvent:
#         
#         buffer = utils.merge_frames(buffer)
#         io_buffer = io.BytesIO()
#     
#         with wave.open(io_buffer, "wb") as wav:
#             wav.setnchannels(buffer.num_channels)
#             wav.setsampwidth(2)  # 16-bit
#             wav.setframerate(buffer.sample_rate)
#             wav.writeframes(buffer.data)
#
#         # wav文件转为mp3格式
#         mp3 = AudioSegment.from_wav(io_buffer)
#         io_buffer = io.BytesIO()
#         mp3.export(io_buffer, format="mp3")
#         
#
#         # 请求接口 对接funASR
#         #url = "http://test/asr"
#
#         files = {'file': ('test.wav', io_buffer.getvalue(), 'audio/wav')}
#         response = requests.post(baseURLW, files=files)
#         resultText = response.json()["result"][0]["text"]
#
#         logger.info(f"response: {resultText}")
#
#         return stt.SpeechEvent(
#             type=stt.SpeechEventType.FINAL_TRANSCRIPT,
#             alternatives=[
#                 stt.SpeechData(text=resultText or "", language=language or "")
#             ],
#         )


class AssistantFnc(llm.FunctionContext):
    """
    The class defines a set of LLM functions that the assistant can execute.
    """
    
    @llm.ai_callable()
    async def _get_timezone_from_llm(self, location: str) -> str:
        """Helper function to get timezone string from LLM."""
        agent = AgentCallContext.get_current().agent
        prompt = f"""Given the location "{location}", return only the exact timezone string from the pytz database (e.g., "America/Toronto", "Asia/Tokyo").
        Return only the timezone string, nothing else. If you're unsure, return "America/Toronto" as default."""
        
        return prompt
        response = await agent.llm.create_completion(prompt)
        timezone_str = response.text.strip().replace('"', '').replace("'", "")
        
        # Validate the timezone string
        if timezone_str not in pytz.all_timezones:
            logger.warning(f"Invalid timezone {timezone_str} returned by LLM for {location}, using default")
            return "America/Toronto"
        
        return timezone_str

    @llm.ai_callable()
    async def get_date(
        self,
        location: Annotated[
            str, llm.TypeInfo(description="The location to get the current date for. If not specified, defaults to Toronto")
        ] = "Toronto",
    ):
        """Called when the user asks about the current date. This function will return the current date for the given location."""
        # Clean the location string of special characters
        location = re.sub(r"[^a-zA-Z0-9]+", " ", location).strip()

        logger.info(f"getting date for {location}")
        
        try:
            # Get timezone string from LLM
            timezone_str = await self._get_timezone_from_llm(location)
            
            # Get the timezone
            tz = pytz.timezone(timezone_str)
            
            # Get current date in the specified timezone
            current_date = datetime.now(tz)
            
            # Format the date string (e.g., "Monday, January 1st, 2024")
            day_suffix = {1: 'st', 2: 'nd', 3: 'rd'}.get(current_date.day % 10, 'th')
            if 11 <= current_date.day <= 13:
                day_suffix = 'th'
                
            formatted_date = current_date.strftime(f"%A, %B %-d{day_suffix}, %Y")
            date_data = f"The current date in {location} is {formatted_date}."
            logger.info(f"date data: {date_data}")
            
        except Exception as e:
            logger.error(f"Error getting date: {str(e)}")
            raise Exception(f"Failed to get date data for {location}: {str(e)}")

        return date_data

    @llm.ai_callable()
    async def get_time(
        self,
        location: Annotated[
            str, llm.TypeInfo(description="The location to get the current time for. If not specified, defaults to Toronto. You first have to get the timezone.")
        ] = "Toronto",
    ):
        """Called when the user asks about the current time. This function will return the current time for the given location."""
        # Clean the location string of special characters
        location = re.sub(r"[^a-zA-Z0-9]+", " ", location).strip()

        # agent = AgentCallContext.get_current().agent

        # if (
        #     not agent.chat_ctx.messages
        #     or agent.chat_ctx.messages[-1].role != "assistant"
        # ):
        #     filler_messages = [
        #         "Let me check the time in {location} for you.",
        #         "I'll find out what time it is in {location} right now.",
        #         "Checking the current time in {location}...",
        #     ]
        #     message = random.choice(filler_messages).format(location=location)
        #     logger.info(f"saying filler message: {message}")
        #     speech_handle = await agent.say(message, add_to_chat_ctx=True)

        logger.info(f"getting time for {location}")
        
        try:
            # Get timezone string from LLM
            timezone_str = await self._get_timezone_from_llm(location)
            
            # Get the timezone
            tz = pytz.timezone(timezone_str)
            
            # Get current time in the specified timezone
            current_time = datetime.now(tz)

            print(current_time)
            
            # Format the time string
            formatted_time = current_time.strftime("%I:%M %p")
            time_data = f"The current time in {location} is {formatted_time}."
            logger.info(f"time data: {time_data}")
            
        except Exception as e:
            logger.error(f"Error getting time: {str(e)}")
            raise Exception(f"Failed to get time data for {location}: {str(e)}")

        return time_data

    @llm.ai_callable()
    async def get_weather(
        self,
        location: Annotated[
            str, llm.TypeInfo(description="The location to get the weather for. If not specified, defaults to Toronto")
        ] = "Toronto",
    ):
        """Called when the user asks about the weather. This function will return the weather for the given location."""
        # Clean the location string of special characters
        location = re.sub(r"[^a-zA-Z0-9]+", " ", location).strip()

        # When a function call is running, there are a couple of options to inform the user
        # that it might take awhile:
        # Option 1: you can use .say filler message immediately after the call is triggered
        # Option 2: you can prompt the agent to return a text response when it's making a function call
        # agent = AgentCallContext.get_current().agent

        # if (
        #     not agent.chat_ctx.messages
        #     or agent.chat_ctx.messages[-1].role != "assistant"
        # ):
        #     # skip if assistant already said something
        #     filler_messages = [
        #         "Let me check the weather in {location} for you.",
        #         "Let me see what the weather is like in {location} right now.",
        #         # LLM will complete this sentence if it is added to the end of the chat context
        #         "The current weather in {location} is ",
        #     ]
        #     message = random.choice(filler_messages).format(location=location)
        #     logger.info(f"saying filler message: {message}")
        #
        #     # NOTE: set add_to_chat_ctx=True will add the message to the end
        #     #   of the chat context of the function call for answer synthesis
        #     speech_handle = await agent.say(message, add_to_chat_ctx=True)  # noqa: F841

        logger.info(f"getting weather for {location}")
        url = f"https://wttr.in/{urllib.parse.quote(location)}?format=%C+%t"
        weather_data = ""
        async with aiohttp.ClientSession() as session:
            async with session.get(url) as response:
                if response.status == 200:
                    # response from the function call is returned to the LLM
                    weather_data = (
                        f"The weather in {location} is {await response.text()}."
                    )
                    logger.info(f"weather data: {weather_data}")
                else:
                    raise Exception(
                        f"Failed to get weather data, status code: {response.status}"
                    )

        # (optional) To wait for the speech to finish before giving results of the function call
        # await speech_handle.join()
        return weather_data

    @llm.ai_callable()
    async def convert_currency(
        self,
        amount: Annotated[float, llm.TypeInfo(description="Amount to convert")],
        from_curr: Annotated[str, llm.TypeInfo(description="Source currency code (e.g., USD)")],
        to_curr: Annotated[str, llm.TypeInfo(description="Target currency code (e.g., EUR)")],
    ):
        """Convert between currencies using real-time exchange rates."""
        logger.info(f"Converting {amount} {from_curr} to {to_curr}")
    
        url = f"https://api.exchangerate.host/convert?from={from_curr}&to={to_curr}&amount={amount}"
        
        async with aiohttp.ClientSession() as session:
            async with session.get(url) as response:
                if response.status == 200:
                    data = await response.json()
                    result = data.get("result")
                    return f"{amount} {from_curr} = {round(result, 2)} {to_curr}"
                else:
                    raise Exception("Currency conversion failed")


def prewarm_process(proc: JobProcess):
    # preload silero VAD in memory to speed up session start
    proc.userdata["vad"] = silero.VAD.load()


async def entrypoint(ctx: JobContext):
    await ctx.connect(auto_subscribe=AutoSubscribe.AUDIO_ONLY)
    fnc_ctx = AssistantFnc()  # create our fnc ctx instance
    initial_chat_ctx = llm.ChatContext().append(
        text=(
            "You are a helpful assistant created by LiveKit. Your interface with users will be voice. "
            "You can provide weather information, current time, current date, currency conversion and other things. "
            "If no location is specified, assume the user is asking about Toronto. "
            # when using option 1, you can suppress from the agent with prompt
            "do not return any text while calling the function."
        ),
        role="system",
    )
    participant = await ctx.wait_for_participant()
    agent = VoicePipelineAgent(
        vad=ctx.proc.userdata["vad"],
        stt=openai.STT(base_url=baseURLW), # stt=deepgram.STT(),
        llm=openai.LLM.with_ollama(
            base_url="http://192.168.10.14:11434/v1",
            model="llama3.1:8b",
        ), # llm=openai.LLM.with_groq(),
        tts=openai.TTS(base_url=baseURL, voice="af"),
        fnc_ctx=fnc_ctx,
        chat_ctx=initial_chat_ctx,
    )

    # Start the assistant. This will automatically publish a microphone track and listen to the participant.
    agent.start(ctx.room, participant)
    # await agent.say(
    #     "Hello! I'm your helpful assistant. I can tell you about the weather, time, or help you with other questions."
    # )


if __name__ == "__main__":
    cli.run_app(
        WorkerOptions(
            entrypoint_fnc=entrypoint,
            prewarm_fnc=prewarm_process,
        ),
    )
