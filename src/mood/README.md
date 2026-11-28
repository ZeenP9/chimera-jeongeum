# 正音 Mood box

Sing·Play 노브 묶음의 **Mood** 페이지다. 잔향 노브 세 개(Dry/Wet · Size · Decay)와 정음 네 조각 스위치, 그리고 스위치 아래 표시 칸으로 이루어진다.

정음키메라는 나중에 성숙한 원본 CHIMERA를 새로 fork한 뒤 이 상자만 옮겨 붙일 계획이다. 그래서 이 폴더는 원본 코드와 맞닿는 곳이 가장 적도록 짜여 있다.

## 폴더 안

| 파일 | 하는 일 | 원본 코드를 아는가 |
|---|---|---|
| `host.ts` | 상자가 원본에서 필요로 하는 것(`MoodHost`)과 그것을 끼우는 `setMoodHost` | 아니오 |
| `store.ts` | 상자의 상태(잔향 값, 네 조각), 스위치 → 읽기 비트(`readingBits`), 위 조각 → 경로(`route`), `resendMood` | 아니오 |
| `room.ts` | 잔향: 임펄스 응답을 만들고 Web Audio로 끼워 넣는 `Room` | 아니오 |
| `MoodPage.tsx` | 노브 세 개와 스위치를 노브 한 페이지로 내놓는 `useMoodPage()` | 원본의 `EncoderPage` 타입만 |
| `JeongeumSwitch.tsx`, `JeongeumLegend.tsx`, `mood.css` | 네 조각 스위치와 표시 칸 | 원본의 CSS 변수(`--white` 등)와 노브 칸 클래스(`.encoder`, `.enc-label`, `.enc-value`, `.enc-unit`)만 |
| `chimera.ts` | **이 fork에 상자를 끼우는 파일.** 원본 엔진을 아는 곳은 여기뿐 | 예 |

## 새 fork에 옮길 때 할 일

1. `src/mood/` 폴더를 통째로 복사한다.
2. `chimera.ts`를 새 원본에 맞게 고친다. `MoodHost`의 네 가지만 채우면 된다.
   - `inPlugin`: 플러그인 안이면 Mood 페이지를 숨긴다(DAW가 잔향을 맡는다).
   - `setRoom`, `setRoomShape`: 잔향 값 전달. 새 원본에 잔향이 없으면 아래 3번처럼 `Room`을 끼운다.
   - `setReadingMarks`: 정음 리더의 읽기 비트 전달. 정음 리더를 아직 붙이지 않았다면 빈 함수로 둬도 상자는 돌아간다.
3. 오디오 출력에 `Room`을 끼운다(`src/audio/AudioEngine.ts`, 이 fork에서 「正音 Mood box」 주석이 달린 네 곳).
   - `readonly room = new Room();`
   - 마스터 다음 출력 직전: `const out = inPlugin ? master : this.room.attach(ctx, master);`, 이후 출력들은 `out`에 잇는다.
   - `setRoom(v)` → `this.room.setMix(v)`, `setRoomShape(s, d)` → `this.room.setShape(s, d)`.
4. 원본 상태 저장소가 새 엔진에 설정을 다시 보내는 곳(이 fork의 `src/ui/store.ts` `sendState`)에 `resendMood()` 한 줄을 넣는다. `chimera.ts`를 import하면 상자가 엔진에 끼워진다.
5. 노브 줄에 상자가 들어갈 자리를 만든다(`src/ui/components/Encoder.tsx`).
   - `EncoderSpec.slot`: 노브 색을 자리와 따로 정한다(Mood는 초록·빨강·흰색).
   - `EncoderPage.extra`: 노브가 아닌 것(스위치)을 줄의 빈자리에 넣는다.
6. 노브 페이지 목록에 `...useMoodPage()`를 넣는다(이 fork: `SingPanel.tsx`의 `SongControls`, `VoiceControls.tsx`).
7. 정음 리더를 쓸 경우, 엔진 쪽 세 곳을 함께 옮긴다.
   - `crates/chimera/src/phonetics/jeongeum.rs`의 `set_marks`, `MARK_I`, `MARK_EU`. `store.ts`의 `READ_*`와 값이 같아야 한다.
   - `crates/chimera-wasm/src/api.rs`의 `"setJeongeumMarks"`
   - `src/wasm/engine.ts`의 `setJeongeumMarks`

## 네 조각과 표시 칸

| 켜진 조각 | 용도 | 표시 칸 | 지금 |
|---|---|---|---|
| 왼쪽 ɨㅣ | 폴란드어 y처럼 날카로운 ɨ (ᆜ = [ɨ]) | ɨ=ᅟᆜ | `READ_I` |
| 오른쪽 ㅡḥ | 힌디어 등 산스크리트계 언어의 정음 표기 | Sanskrit | `READ_EU` |
| 왼쪽 + 오른쪽 | 프랑스어를 정음으로 적은 글 (두 기능을 겹치는 것이 아닌 별개 모드) | 프랑스 국기 | 규칙 미정, 중립으로 읽음 |
| 아래 ㆆع | 아랍어를 정음으로 적은 글 | حَبِيبِي (다른 조각이 켜져도 먼저 보임) | 규칙 미정 |
| 위 ⊙ | 경로 전환(아래) | 태극 (다른 조각이 켜지면 물러남) | `route()`만 있음 |

모두 꺼진 상태가 기본이고, 기본 상태에서 주요 언어가 제대로 읽혀야 한다. 스위치는 대부분의 사용자가 건드리지 않는 미세 조정이다.

## 위 조각: 두 가지 경로

`route(marks)`가 돌려준다.

- **`"ipa"` (꺼짐, 기본)**: 입력창의 외국어는 원본의 언어 처리기가 IPA로 바꾸고, 엔진은 그 IPA로 소리 낸다. 정음은 그 IPA를 캔버스에 정음으로 보여 주기만 한다(백댄서).
- **`"jeongeum"` (켜짐)**: 입력창의 글을 먼저 정음으로 바꾸고, 그 정음을 정음 리더가 IPA로 읽어 엔진에 보낸다.

아직 연결하지 않았다. 연결할 때는 원본이 가사를 음절로 바꾸는 곳(이 fork: `lyricsToSyllables`를 부르는 곳)에서 `route(useMood.getState().marks)`를 보고 경로를 고르면 된다. 상자 쪽은 바꿀 것이 없다.

## 태극 로고

괘 막대의 경로(`JeongeumLegend.tsx`의 `GWAE`)는 `tools/gen-jeongeum-taegeuk.py`로 만든다. 태극 반지름, 막대가 끝나는 선, 연마 반지름을 바꾸면 다시 만든다.
