# Sacho로 변경 기록 작성하기

이 저장소는 [Sacho](https://sacho.dev/)로 사용자에게 필요한 변경 설명을 관리합니다. 이번에 적용한 버전은 **0.3.2**이며, `mise.toml`에 고정되어 있습니다.

## 어떤 파일을 수정하나요?

| 파일·폴더 | 역할 |
| --- | --- |
| [`changes.d/`](../changes.d/) | 아직 릴리스하지 않은 변경을 주제별 Markdown 파일로 작성하는 곳입니다. 이 파일들이 원본입니다. |
| [`CHANGES.md`](../CHANGES.md) | Sacho가 메모를 모아 보여 주는 변경 기록입니다. 릴리스 전 영역은 직접 편집하지 않습니다. |
| [`sacho.toml`](../sacho.toml) | 변경 기록 경로, 메모 폴더, 검사할 소스 경로를 설정합니다. |
| [`mise.toml`](../mise.toml) | Sacho 버전과 자주 사용하는 명령을 정의합니다. |

사용자에게 무엇이 달라졌는지, 업그레이드할 때 무엇을 수정해야 하는지 설명하세요. 구현 순서나 커밋 목록을 그대로 옮길 필요는 없습니다. 메모 파일 이름은 이슈 번호 대신 `borrowed-encoding.md`처럼 변경 주제를 사용합니다.

## 처음 준비할 때

[mise](https://mise.jdx.dev/)가 설치된 환경에서 저장소 루트에서 실행합니다.

```sh
mise trust
mise install
```

mise 없이 사용하려면 `cargo install sacho --version 0.3.2 --locked`로 설치하고 아래 명령에서 `mise exec --`를 빼면 됩니다.

Git 병합 드라이버가 필요한 개발자는 `mise exec -- sacho init --no-interactive`로 자신의 체크아웃에 연동할 수 있습니다. 커밋 훅도 설치하려면 `--install-hook`을 추가합니다. 저장소 설정과 CI 검사는 훅 설치 없이도 동작합니다.

## 변경을 기록할 때

1. `mise exec -- sacho add topic-name`으로 메모 파일을 만듭니다.
2. 명령이 알려 준 `changes.d/` 파일을 편집합니다. 최상위 불릿 목록 하나를 사용하며, 보통 항목 하나에 변경 하나를 설명합니다.
3. 같은 기능을 릴리스 전에 다시 고쳤다면 기존 메모를 수정합니다. 사용자에게 공개되지 않은 중간 상태를 새 항목으로 계속 쌓지 않습니다.
4. 다음 명령으로 형식을 정리하고 결과를 확인합니다.

```sh
mise run changelog:fmt
mise run changelog:preview
mise run changelog:check
```

`fmt`는 메모 형식과 `CHANGES.md`의 릴리스 전 영역을 함께 갱신합니다. 생성된 내용만 오래된 상태라면 `mise exec -- sacho sync`로 다시 맞출 수 있습니다.

`src/**` 또는 `Cargo.toml`을 바꾸는 경우 **소스 변경과 메모를 같은 커밋에 포함해야 합니다.** CI는 PR의 기준 커밋 이후를 커밋별로 검사하므로, 메모를 나중에 별도 커밋으로 추가해도 앞선 커밋의 누락이 해결되지는 않습니다. 사용자에게 영향이 없는 유지보수 변경은 커밋 메시지 끝에 `Changelog: none` 트레일러를 명시해 제외할 수 있습니다.

로컬에서도 `mise exec -- sacho check --base <revision>`으로 이미 커밋한 변경을 비교할 수 있습니다. 커밋하기 전 스테이징한 변경은 `mise exec -- sacho check --staged`로 검사합니다. 옵션 없는 `sacho check`는 메모 형식과 생성 결과만 확인하며 소스 변경의 메모 누락까지 검사하지는 않습니다.

## 릴리스할 때

이번 도입에서는 다음 릴리스 버전을 임의로 정하지 않았습니다. 버전을 결정한 뒤 미리보기와 변경 내용을 검토하고 다음을 실행합니다.

```sh
mise exec -- sacho release <version>
```

Sacho는 메모를 해당 버전의 변경 기록으로 확정하고, 사용한 메모 파일을 삭제합니다. 이 상태를 커밋하고 태그를 붙인 뒤, 다음 개발 주기는 `mise exec -- sacho next <next-version>`으로 별도로 시작합니다.

이 명령은 크레이트를 배포하지 않습니다. 패키지 버전 수정과 배포는 기존 릴리스 절차에서 진행해야 합니다. 이번 도입은 앞으로의 변경부터 관리하며 과거 릴리스 기록을 새로 만들어 넣지 않습니다.
