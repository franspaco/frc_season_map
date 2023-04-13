# From https://github.com/the-blue-alliance/the-blue-alliance/blob/master/consts/event_type.py#L2
class EventType:
    REGIONAL = 0
    DISTRICT = 1
    DISTRICT_CMP = 2
    CMP_DIVISION = 3
    CMP_FINALS = 4
    DISTRICT_CMP_DIVISION = 5
    FOC = 6
    REMOTE = 7

    OFFSEASON = 99
    PRESEASON = 100
    UNLABLED = -1

    @staticmethod
    def isChampionship(val: int):
        return val == EventType.CMP_DIVISION or val == EventType.CMP_FINALS
